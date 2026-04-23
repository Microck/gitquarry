use crate::cli::SearchArgs;
use crate::error::{AppError, AppResult};
use crate::model::{
    DiscoveryDepth, OutputFormat, ProgressMode, RankMode, RetrievalMode, ScoreWeights, SearchSort,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchPlan {
    pub query: Option<String>,
    pub compiled_query: String,
    pub mode: RetrievalMode,
    pub rank: RankMode,
    pub sort: SearchSort,
    pub depth: DiscoveryDepth,
    pub format: OutputFormat,
    pub limit: usize,
    pub readme: bool,
    pub explain: bool,
    pub weights: ScoreWeights,
    pub concurrency: usize,
    pub progress: ProgressMode,
    pub post_filters: PostFilters,
    pub native_query_present: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PostFilters {
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
}

pub fn build_search_plan(
    args: &SearchArgs,
    default_format: OutputFormat,
    default_limit: usize,
    default_progress: ProgressMode,
    now: DateTime<Utc>,
) -> AppResult<SearchPlan> {
    validate_flag_rules(args)?;

    let mode = args.mode.unwrap_or(RetrievalMode::Native);
    let rank = match (mode, args.rank) {
        (RetrievalMode::Discover, None) => RankMode::Blended,
        (_, Some(rank)) => rank,
        _ => RankMode::Native,
    };
    let depth = args.depth.unwrap_or(DiscoveryDepth::Balanced);
    let limit = args.limit.unwrap_or(default_limit).max(1);
    let format = args.format.unwrap_or(default_format);
    let progress = args.progress.unwrap_or(default_progress);
    let concurrency = args.concurrency.unwrap_or(1).max(1);
    let weights = ScoreWeights {
        query: args.weight_query.unwrap_or(1.0),
        activity: args.weight_activity.unwrap_or(1.0),
        quality: args.weight_quality.unwrap_or(1.0),
    };

    if rank == RankMode::Blended
        && weights.query == 0.0
        && weights.activity == 0.0
        && weights.quality == 0.0
    {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            "all blended weights cannot be zero",
        ));
    }

    let qualifiers = compile_qualifiers(args, now)?;
    let base_query = args
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if mode == RetrievalMode::Native && base_query.is_none() {
        return Err(AppError::new(
            "E_QUERY_REQUIRED",
            "empty query is invalid outside explicit discovery mode",
        ));
    }

    let compiled_query = match (base_query, qualifiers.is_empty(), mode) {
        (Some(query), true, _) => query.to_string(),
        (Some(query), false, _) => format!("{query} {}", qualifiers.join(" ")),
        (None, false, _) => qualifiers.join(" "),
        (None, true, RetrievalMode::Discover) => "stars:>=1".to_string(),
        (None, true, RetrievalMode::Native) => unreachable!(),
    };

    Ok(SearchPlan {
        query: base_query.map(ToOwned::to_owned),
        compiled_query,
        mode,
        rank,
        sort: args.sort,
        depth,
        format,
        limit,
        readme: args.readme,
        explain: args.explain,
        weights,
        concurrency,
        progress,
        post_filters: PostFilters {
            updated_after: absolute_or_relative(&args.updated_after, &args.updated_within, now)?,
            updated_before: parse_date_opt(args.updated_before.as_deref())?.map(as_utc_end),
        },
        native_query_present: base_query.is_some(),
    })
}

pub(crate) fn compiled_query_has_qualifier(query: &str, qualifier: &str) -> bool {
    let pattern = format!(r"(?i)(?:^|\s){}\s*:", regex::escape(qualifier));
    Regex::new(&pattern).unwrap().is_match(query)
}

fn validate_flag_rules(args: &SearchArgs) -> AppResult<()> {
    detect_raw_query_conflicts(args)?;

    if args.user.is_some() && args.org.is_some() {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            "--user cannot be combined with --org",
        ));
    }

    validate_range("stars", args.min_stars, args.max_stars)?;
    validate_range("forks", args.min_forks, args.max_forks)?;
    validate_range("size", args.min_size, args.max_size)?;

    validate_date_pair(
        "--created-after",
        "--created-before",
        args.created_after.as_deref(),
        args.created_before.as_deref(),
    )?;
    validate_date_pair(
        "--updated-after",
        "--updated-before",
        args.updated_after.as_deref(),
        args.updated_before.as_deref(),
    )?;
    validate_date_pair(
        "--pushed-after",
        "--pushed-before",
        args.pushed_after.as_deref(),
        args.pushed_before.as_deref(),
    )?;

    validate_absolute_relative(
        "--created-after",
        "--created-within",
        args.created_after.as_deref(),
        args.created_within.as_deref(),
    )?;
    validate_absolute_relative(
        "--updated-after",
        "--updated-within",
        args.updated_after.as_deref(),
        args.updated_within.as_deref(),
    )?;
    validate_absolute_relative(
        "--pushed-after",
        "--pushed-within",
        args.pushed_after.as_deref(),
        args.pushed_within.as_deref(),
    )?;

    if args.depth.is_some() && args.mode != Some(RetrievalMode::Discover) {
        return Err(AppError::new(
            "E_FLAG_REQUIRES_MODE",
            "--depth requires --mode discover",
        ));
    }

    if args.explain && args.mode != Some(RetrievalMode::Discover) {
        return Err(AppError::new(
            "E_FLAG_REQUIRES_MODE",
            "--explain requires --mode discover",
        ));
    }

    if let Some(rank) = args.rank
        && rank != RankMode::Native
        && args.mode != Some(RetrievalMode::Discover)
    {
        return Err(AppError::new(
            "E_FLAG_REQUIRES_MODE",
            format!("--rank {:?} requires --mode discover", rank).to_ascii_lowercase(),
        ));
    }

    if (args.weight_query.is_some()
        || args.weight_activity.is_some()
        || args.weight_quality.is_some())
        && args.rank != Some(RankMode::Blended)
    {
        return Err(AppError::new(
            "E_FLAG_REQUIRES_MODE",
            "weight flags require --rank blended",
        ));
    }

    validate_weight("--weight-query", args.weight_query)?;
    validate_weight("--weight-activity", args.weight_activity)?;
    validate_weight("--weight-quality", args.weight_quality)?;

    if let Some(concurrency) = args.concurrency {
        if concurrency == 0 {
            return Err(AppError::new(
                "E_FLAG_CONFLICT",
                "--concurrency must be at least 1",
            ));
        }
        if args.mode != Some(RetrievalMode::Discover) && !args.readme {
            return Err(AppError::new(
                "E_FLAG_REQUIRES_MODE",
                "--concurrency requires --mode discover or --readme",
            ));
        }
    }

    Ok(())
}

fn compile_qualifiers(args: &SearchArgs, now: DateTime<Utc>) -> AppResult<Vec<String>> {
    let mut qualifiers = Vec::new();

    if let Some(user) = &args.user {
        qualifiers.push(format!("user:{user}"));
    }
    if let Some(org) = &args.org {
        qualifiers.push(format!("org:{org}"));
    }
    if let Some(archived) = args.archived {
        qualifiers.push(format!("archived:{}", archived.as_bool()));
    }
    if let Some(template) = args.template {
        qualifiers.push(format!("template:{}", template.as_bool()));
    }
    if let Some(fork) = args.fork {
        qualifiers.push(format!("fork:{}", fork.qualifier()));
    }

    qualifiers.extend(
        args.language
            .iter()
            .map(|value| format!("language:{value}")),
    );
    qualifiers.extend(args.topic.iter().map(|value| format!("topic:{value}")));
    qualifiers.extend(args.license.iter().map(|value| format!("license:{value}")));

    push_numeric_range(&mut qualifiers, "stars", args.min_stars, args.max_stars);
    push_numeric_range(&mut qualifiers, "forks", args.min_forks, args.max_forks);
    push_numeric_range(&mut qualifiers, "size", args.min_size, args.max_size);

    push_date_range(
        &mut qualifiers,
        "created",
        absolute_or_relative(&args.created_after, &args.created_within, now)?,
        parse_date_opt(args.created_before.as_deref())?.map(as_utc_end),
    );
    push_date_range(
        &mut qualifiers,
        "pushed",
        absolute_or_relative(&args.pushed_after, &args.pushed_within, now)?,
        parse_date_opt(args.pushed_before.as_deref())?.map(as_utc_end),
    );

    Ok(qualifiers)
}

fn push_numeric_range(target: &mut Vec<String>, label: &str, min: Option<u64>, max: Option<u64>) {
    match (min, max) {
        (Some(min), Some(max)) => target.push(format!("{label}:{min}..{max}")),
        (Some(min), None) => target.push(format!("{label}:>={min}")),
        (None, Some(max)) => target.push(format!("{label}:<={max}")),
        (None, None) => {}
    }
}

fn push_date_range(
    target: &mut Vec<String>,
    label: &str,
    after: Option<DateTime<Utc>>,
    before: Option<DateTime<Utc>>,
) {
    let fmt = |value: DateTime<Utc>| value.format("%Y-%m-%d").to_string();
    match (after, before) {
        (Some(after), Some(before)) => {
            target.push(format!("{label}:{}..{}", fmt(after), fmt(before)))
        }
        (Some(after), None) => target.push(format!("{label}:>={}", fmt(after))),
        (None, Some(before)) => target.push(format!("{label}:<={}", fmt(before))),
        (None, None) => {}
    }
}

fn validate_range(label: &str, min: Option<u64>, max: Option<u64>) -> AppResult<()> {
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            format!("min {label} cannot be greater than max {label}"),
        ));
    }
    Ok(())
}

fn validate_weight(flag: &str, value: Option<f64>) -> AppResult<()> {
    if let Some(value) = value
        && !(0.0..=3.0).contains(&value)
    {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            format!("{flag} must be within 0.0..=3.0"),
        ));
    }
    Ok(())
}

fn validate_date_pair(
    after_flag: &str,
    before_flag: &str,
    after: Option<&str>,
    before: Option<&str>,
) -> AppResult<()> {
    let after = parse_date_opt(after)?;
    let before = parse_date_opt(before)?;
    if let (Some(after), Some(before)) = (after, before)
        && after > before
    {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            format!("{after_flag} cannot be later than {before_flag}"),
        ));
    }
    Ok(())
}

fn validate_absolute_relative(
    absolute_flag: &str,
    relative_flag: &str,
    absolute: Option<&str>,
    relative: Option<&str>,
) -> AppResult<()> {
    if absolute.is_some() && relative.is_some() {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            format!("{absolute_flag} cannot be combined with {relative_flag}"),
        ));
    }
    Ok(())
}

fn detect_raw_query_conflicts(args: &SearchArgs) -> AppResult<()> {
    let query = args.query.as_deref().unwrap_or("").trim();
    if query.is_empty() {
        return Ok(());
    }

    let overlaps = [
        ("language", !args.language.is_empty()),
        ("topic", !args.topic.is_empty()),
        ("license", !args.license.is_empty()),
        (
            "stars",
            args.min_stars.is_some() || args.max_stars.is_some(),
        ),
        (
            "forks",
            args.min_forks.is_some() || args.max_forks.is_some(),
        ),
        ("size", args.min_size.is_some() || args.max_size.is_some()),
        (
            "created",
            args.created_after.is_some()
                || args.created_before.is_some()
                || args.created_within.is_some(),
        ),
        (
            "updated",
            args.updated_after.is_some()
                || args.updated_before.is_some()
                || args.updated_within.is_some(),
        ),
        (
            "pushed",
            args.pushed_after.is_some()
                || args.pushed_before.is_some()
                || args.pushed_within.is_some(),
        ),
        ("user", args.user.is_some()),
        ("org", args.org.is_some()),
        ("archived", args.archived.is_some()),
        ("template", args.template.is_some()),
        ("fork", args.fork.is_some()),
    ];

    for (qualifier, active) in overlaps {
        if active && compiled_query_has_qualifier(query, qualifier) {
            return Err(AppError::new(
                "E_FLAG_CONFLICT",
                format!(
                    "raw query qualifier {qualifier}: conflicts with overlapping structured flags"
                ),
            ));
        }
    }

    Ok(())
}

fn absolute_or_relative(
    absolute: &Option<String>,
    relative: &Option<String>,
    now: DateTime<Utc>,
) -> AppResult<Option<DateTime<Utc>>> {
    if let Some(value) = absolute {
        return Ok(parse_date_opt(Some(value.as_str()))?.map(as_utc_start));
    }

    if let Some(value) = relative {
        let duration = parse_relative_duration(value)?;
        return Ok(Some(now - duration));
    }

    Ok(None)
}

fn parse_date_opt(value: Option<&str>) -> AppResult<Option<NaiveDate>> {
    value
        .map(|item| {
            NaiveDate::parse_from_str(item, "%Y-%m-%d").map_err(|err| {
                AppError::with_detail(
                    "E_FLAG_CONFLICT",
                    "invalid date; expected YYYY-MM-DD",
                    err.to_string(),
                )
            })
        })
        .transpose()
}

fn parse_relative_duration(value: &str) -> AppResult<Duration> {
    let Some(unit) = value.chars().last() else {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            "relative duration must not be empty",
        ));
    };
    let number = value[..value.len() - 1].parse::<i64>().map_err(|err| {
        AppError::with_detail(
            "E_FLAG_CONFLICT",
            "invalid relative duration",
            err.to_string(),
        )
    })?;

    let duration = match unit {
        'h' => Duration::hours(number),
        'd' => Duration::days(number),
        'w' => Duration::weeks(number),
        'm' => Duration::days(number * 30),
        'y' => Duration::days(number * 365),
        _ => {
            return Err(AppError::new(
                "E_FLAG_CONFLICT",
                "relative duration must end with h, d, w, m, or y",
            ));
        }
    };

    Ok(duration)
}

fn as_utc_start(date: NaiveDate) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc)
}

fn as_utc_end(date: NaiveDate) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(date.and_hms_opt(23, 59, 59).unwrap(), Utc)
}

pub fn apply_post_filters<'a>(
    repos: impl IntoIterator<Item = &'a crate::model::Repository>,
    filters: &PostFilters,
) -> Vec<crate::model::Repository> {
    repos
        .into_iter()
        .filter(|repo| {
            filters
                .updated_after
                .map(|value| repo.updated_at >= value)
                .unwrap_or(true)
        })
        .filter(|repo| {
            filters
                .updated_before
                .map(|value| repo.updated_at <= value)
                .unwrap_or(true)
        })
        .cloned()
        .collect()
}

pub fn discovery_target(depth: DiscoveryDepth, limit: usize) -> usize {
    match depth {
        DiscoveryDepth::Quick => (limit * 3).clamp(25, 100),
        DiscoveryDepth::Balanced => (limit * 5).clamp(50, 200),
        DiscoveryDepth::Deep => (limit * 8).clamp(100, 400),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_post_filters, build_search_plan, compiled_query_has_qualifier, discovery_target,
    };
    use crate::cli::SearchArgs;
    use crate::model::{
        DiscoveryDepth, OutputFormat, Owner, ProgressMode, RankMode, Repository, RetrievalMode,
        SearchSort,
    };
    use chrono::{TimeZone, Utc};

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 22, 12, 0, 0).unwrap()
    }

    fn baseline_args() -> SearchArgs {
        SearchArgs {
            query: Some("rust cli".to_string()),
            mode: None,
            rank: None,
            sort: SearchSort::BestMatch,
            depth: None,
            format: None,
            limit: None,
            user: None,
            org: None,
            archived: None,
            template: None,
            fork: None,
            language: Vec::new(),
            topic: Vec::new(),
            license: Vec::new(),
            min_stars: None,
            max_stars: None,
            min_forks: None,
            max_forks: None,
            min_size: None,
            max_size: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            pushed_after: None,
            pushed_before: None,
            created_within: None,
            updated_within: None,
            pushed_within: None,
            readme: false,
            explain: false,
            weight_query: None,
            weight_activity: None,
            weight_quality: None,
            concurrency: None,
            progress: None,
        }
    }

    fn repo(name: &str, updated_at: chrono::DateTime<Utc>) -> Repository {
        Repository {
            name: name.to_string(),
            full_name: format!("owner/{name}"),
            html_url: format!("https://example.com/{name}"),
            description: Some(format!("{name} description")),
            stargazers_count: 1,
            forks_count: 1,
            language: Some("Rust".to_string()),
            topics: vec!["cli".to_string()],
            license: None,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at,
            pushed_at: updated_at,
            archived: false,
            is_template: false,
            fork: false,
            open_issues_count: Some(0),
            owner: Owner {
                login: "owner".to_string(),
            },
            readme: None,
            latest_release: None,
            contributor_count: Some(1),
            explain: None,
        }
    }

    #[test]
    fn native_search_requires_query() {
        let mut args = baseline_args();
        args.query = None;
        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();
        assert_eq!(err.code, "E_QUERY_REQUIRED");
    }

    #[test]
    fn discover_defaults_to_blended_rank() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();
        assert_eq!(plan.rank, RankMode::Blended);
    }

    #[test]
    fn rejects_raw_query_conflict() {
        let mut args = baseline_args();
        args.query = Some("language:rust cli".to_string());
        args.language.push("go".to_string());
        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();
        assert_eq!(err.code, "E_FLAG_CONFLICT");
    }

    #[test]
    fn rejects_non_native_rank_without_discover_mode() {
        let mut args = baseline_args();
        args.rank = Some(RankMode::Blended);
        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();
        assert_eq!(err.code, "E_FLAG_REQUIRES_MODE");
    }

    #[test]
    fn rejects_out_of_range_weights() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.rank = Some(RankMode::Blended);
        args.weight_query = Some(3.5);
        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();
        assert_eq!(err.code, "E_FLAG_CONFLICT");
    }

    #[test]
    fn detects_qualifier_with_case_and_spacing_variants() {
        assert!(compiled_query_has_qualifier("rust cli Stars:>10", "stars"));
        assert!(compiled_query_has_qualifier(
            "rust cli Stars : >10",
            "stars"
        ));
        assert!(!compiled_query_has_qualifier(
            "rust cli start:here",
            "stars"
        ));
    }

    #[test]
    fn build_search_plan_compiles_structured_qualifiers() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.user = Some("microck".to_string());
        args.language = vec!["Rust".to_string(), "TypeScript".to_string()];
        args.topic = vec!["cli".to_string()];
        args.license = vec!["mit".to_string()];
        args.min_stars = Some(100);
        args.max_forks = Some(50);
        args.created_after = Some("2024-01-01".to_string());
        args.pushed_before = Some("2026-04-20".to_string());

        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();

        assert_eq!(
            plan.compiled_query,
            "rust cli user:microck language:Rust language:TypeScript topic:cli license:mit stars:>=100 forks:<=50 created:>=2024-01-01 pushed:<=2026-04-20"
        );
        assert!(plan.native_query_present);
    }

    #[test]
    fn discover_without_query_and_filters_uses_seed_query() {
        let mut args = baseline_args();
        args.query = None;
        args.mode = Some(RetrievalMode::Discover);

        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();

        assert_eq!(plan.compiled_query, "stars:>=1");
        assert!(!plan.native_query_present);
    }

    #[test]
    fn updated_filters_stay_out_of_compiled_query_and_become_post_filters() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.updated_after = Some("2026-04-01".to_string());
        args.updated_before = Some("2026-04-20".to_string());

        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();

        assert_eq!(plan.compiled_query, "rust cli");
        assert_eq!(
            plan.post_filters.updated_after,
            Some(Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap())
        );
        assert_eq!(
            plan.post_filters.updated_before,
            Some(Utc.with_ymd_and_hms(2026, 4, 20, 23, 59, 59).unwrap())
        );
    }

    #[test]
    fn relative_dates_are_resolved_from_fixed_now() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.created_within = Some("30d".to_string());
        args.updated_within = Some("12h".to_string());

        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();

        assert!(plan.compiled_query.contains("created:>=2026-03-23"));
        assert_eq!(
            plan.post_filters.updated_after,
            Some(Utc.with_ymd_and_hms(2026, 4, 22, 0, 0, 0).unwrap())
        );
    }

    #[test]
    fn rejects_user_and_org_together() {
        let mut args = baseline_args();
        args.user = Some("microck".to_string());
        args.org = Some("micr".to_string());

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(err.message.contains("--user cannot be combined with --org"));
    }

    #[test]
    fn rejects_inverted_numeric_ranges() {
        let mut args = baseline_args();
        args.min_stars = Some(200);
        args.max_stars = Some(100);

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(
            err.message
                .contains("min stars cannot be greater than max stars")
        );
    }

    #[test]
    fn rejects_inverted_date_ranges() {
        let mut args = baseline_args();
        args.created_after = Some("2026-04-21".to_string());
        args.created_before = Some("2026-04-20".to_string());

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(
            err.message
                .contains("--created-after cannot be later than --created-before")
        );
    }

    #[test]
    fn rejects_absolute_and_relative_date_mix() {
        let mut args = baseline_args();
        args.pushed_after = Some("2026-04-01".to_string());
        args.pushed_within = Some("30d".to_string());

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(
            err.message
                .contains("--pushed-after cannot be combined with --pushed-within")
        );
    }

    #[test]
    fn rejects_invalid_relative_duration_units() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.created_within = Some("3q".to_string());

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(
            err.message
                .contains("relative duration must end with h, d, w, m, or y")
        );
    }

    #[test]
    fn rejects_invalid_date_format() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.created_after = Some("04/01/2026".to_string());

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(err.message.contains("invalid date; expected YYYY-MM-DD"));
    }

    #[test]
    fn readme_allows_concurrency_outside_discover_mode() {
        let mut args = baseline_args();
        args.concurrency = Some(2);
        args.readme = true;

        let plan = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap();

        assert_eq!(plan.concurrency, 2);
    }

    #[test]
    fn blended_rank_rejects_all_zero_weights() {
        let mut args = baseline_args();
        args.mode = Some(RetrievalMode::Discover);
        args.rank = Some(RankMode::Blended);
        args.weight_query = Some(0.0);
        args.weight_activity = Some(0.0);
        args.weight_quality = Some(0.0);

        let err = build_search_plan(
            &args,
            OutputFormat::Pretty,
            10,
            ProgressMode::Auto,
            fixed_now(),
        )
        .unwrap_err();

        assert_eq!(err.code, "E_FLAG_CONFLICT");
        assert!(err.message.contains("all blended weights cannot be zero"));
    }

    #[test]
    fn apply_post_filters_respects_updated_range_bounds() {
        let repos = vec![
            repo("early", Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap()),
            repo(
                "middle",
                Utc.with_ymd_and_hms(2026, 4, 10, 0, 0, 0).unwrap(),
            ),
            repo("late", Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap()),
        ];
        let filters = super::PostFilters {
            updated_after: Some(Utc.with_ymd_and_hms(2026, 4, 5, 0, 0, 0).unwrap()),
            updated_before: Some(Utc.with_ymd_and_hms(2026, 4, 20, 23, 59, 59).unwrap()),
        };

        let filtered = apply_post_filters(&repos, &filters);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "middle");
    }

    #[test]
    fn discovery_target_honors_depth_caps_and_floors() {
        assert_eq!(discovery_target(DiscoveryDepth::Quick, 1), 25);
        assert_eq!(discovery_target(DiscoveryDepth::Balanced, 60), 200);
        assert_eq!(discovery_target(DiscoveryDepth::Deep, 1), 100);
    }
}
