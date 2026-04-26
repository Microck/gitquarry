use crate::cli::{
    AuthArgs, AuthCommand, AuthLoginArgs, Cli, Command, ConfigArgs, ConfigCommand, InspectArgs,
    SearchArgs,
};
use crate::config::ConfigBundle;
use crate::credential::{
    delete_token, env_credential_source, resolve_token, save_token, saved_credential_source,
};
use crate::error::{AppError, AppResult};
use crate::github::GitHubClient;
use crate::host::{HostContext, normalize_host};
use crate::model::{
    ColorPreference, CredentialSource, InspectOutput, OutputFormat, ProgressMode, RankMode,
    Repository, RetrievalMode, SearchOutput, SearchSort,
};
use crate::output::{progress, write_inspect, write_line, write_search};
use crate::query::{
    apply_post_filters, build_search_plan, compiled_query_has_qualifier, discovery_target,
};
use crate::score::rerank;
use chrono::{Duration, Utc};
use clap::{CommandFactory, Parser, error::ErrorKind};
use clap_complete::generate;
use rayon::prelude::*;
use std::io::{self, IsTerminal, Read, Write};
use std::process::exit;

pub fn main_entry() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                error.print().map_err(|err| {
                    AppError::with_detail(
                        "E_OUTPUT",
                        "failed to print clap output",
                        err.to_string(),
                    )
                })?;
                return Ok(());
            }
            _ => {
                return Err(AppError::new(
                    "E_FLAG_PARSE",
                    error.to_string().trim().to_string(),
                ));
            }
        },
    };

    if let Some(shell) = cli.generate_completion {
        let mut command = Cli::command();
        generate(
            shell.to_clap_shell(),
            &mut command,
            "gitquarry",
            &mut io::stdout(),
        );
        return Ok(());
    }

    let config = ConfigBundle::load()?;
    match &cli.command {
        Some(Command::Search(args)) => search_command(&cli, &config, args),
        Some(Command::Inspect(args)) => inspect_command(&cli, &config, args),
        Some(Command::Auth(args)) => auth_command(&cli, &config, args),
        Some(Command::Config(args)) => config_command(&config, args),
        Some(Command::Version) => write_line(&format!("gitquarry {}", env!("CARGO_PKG_VERSION"))),
        None => {
            let mut command = Cli::command();
            command.print_help().map_err(|err| {
                AppError::with_detail("E_OUTPUT", "failed to render help", err.to_string())
            })?;
            io::stdout().write_all(b"\n").map_err(|err| {
                AppError::with_detail("E_OUTPUT", "failed to write help newline", err.to_string())
            })
        }
    }
}

fn search_command(cli: &Cli, config: &ConfigBundle, args: &SearchArgs) -> AppResult<()> {
    let host = resolve_host(cli, config)?;
    let plan = build_search_plan(
        args,
        config.data.format.unwrap_or(OutputFormat::Pretty),
        config.data.limit.unwrap_or(10),
        config.data.progress.unwrap_or(ProgressMode::Auto),
        Utc::now(),
    )?;
    let token = resolve_token(&host, config)?;
    let client = GitHubClient::new(host.api_base.clone(), token.token)?;
    let show_progress = progress_enabled(plan.progress);

    progress(
        show_progress,
        format!("searching host={} mode={:?}", host.web_host, plan.mode),
    );
    let (mut repos, total_count) = match plan.mode {
        RetrievalMode::Native => {
            let page = client.search_repositories(
                &plan.compiled_query,
                plan.sort.as_github_value(),
                plan.limit.min(100),
                1,
            )?;
            (page.items, page.total_count)
        }
        RetrievalMode::Discover => {
            let repos = discovery_search(&client, &plan, show_progress)?;
            let total_count = repos.len();
            (repos, total_count)
        }
    };

    repos = apply_post_filters(repos.iter(), &plan.post_filters);

    if repos.is_empty() {
        let output = SearchOutput {
            host: host.web_host,
            mode: plan.mode,
            rank: plan.rank,
            query: plan.query,
            compiled_query: plan.compiled_query,
            limit: plan.limit,
            total_count,
            items: Vec::new(),
        };
        return write_search(
            &output,
            plan.format,
            config.data.color.unwrap_or(ColorPreference::Auto),
        );
    }

    let concurrency = plan.concurrency.max(1);
    let needs_metadata_enrichment =
        plan.mode == RetrievalMode::Discover || plan.rank != RankMode::Native;
    if needs_metadata_enrichment {
        enrich_metadata(&client, &mut repos, concurrency, show_progress)?;
    }

    if plan.rank != RankMode::Native {
        rerank(
            &mut repos,
            plan.rank,
            plan.query.as_deref(),
            &plan.weights,
            plan.explain,
            Utc::now(),
        );
    }

    if plan.readme {
        progress(show_progress, "enriching readme window");
        enrich_readme_window(&client, &mut repos, plan.limit, concurrency)?;
        if plan.rank != RankMode::Native {
            rerank(
                &mut repos,
                plan.rank,
                plan.query.as_deref(),
                &plan.weights,
                plan.explain,
                Utc::now(),
            );
        }
    }

    if plan.mode == RetrievalMode::Discover && plan.rank == RankMode::Native {
        sort_native_results(&mut repos, plan.sort);
    }

    repos.truncate(plan.limit);

    let output = SearchOutput {
        host: host.web_host,
        mode: plan.mode,
        rank: plan.rank,
        query: plan.query,
        compiled_query: plan.compiled_query,
        limit: plan.limit,
        total_count,
        items: repos,
    };

    write_search(
        &output,
        plan.format,
        config.data.color.unwrap_or(ColorPreference::Auto),
    )
}

fn inspect_command(cli: &Cli, config: &ConfigBundle, args: &InspectArgs) -> AppResult<()> {
    let host = resolve_host(cli, config)?;
    let show_progress = progress_enabled(
        args.progress
            .unwrap_or(config.data.progress.unwrap_or(ProgressMode::Auto)),
    );

    let (owner, repo) = parse_owner_repo(&args.repository)?;
    let token = resolve_token(&host, config)?;
    let client = GitHubClient::new(host.api_base.clone(), token.token)?;
    progress(show_progress, format!("inspecting {owner}/{repo}"));
    let mut repository = client.repository(&owner, &repo)?;
    repository.latest_release = client.latest_release(&owner, &repo)?;
    repository.contributor_count = client.contributor_count(&owner, &repo)?;

    if args.readme {
        progress(show_progress, "fetching readme");
        repository.readme = client.readme(&owner, &repo)?;
    }

    let output = InspectOutput {
        host: host.web_host,
        repository,
    };
    write_inspect(
        &output,
        args.format
            .unwrap_or(config.data.format.unwrap_or(OutputFormat::Pretty)),
        config.data.color.unwrap_or(ColorPreference::Auto),
    )
}

fn auth_command(cli: &Cli, config: &ConfigBundle, args: &AuthArgs) -> AppResult<()> {
    let host = resolve_host(cli, config)?;
    match &args.command {
        AuthCommand::Login(login) => auth_login(config, &host, login),
        AuthCommand::Status => auth_status(config, &host),
        AuthCommand::Logout => auth_logout(config, &host),
    }
}

fn config_command(config: &ConfigBundle, args: &ConfigArgs) -> AppResult<()> {
    match args.command {
        ConfigCommand::Path => write_line(&config.paths.config_file.display().to_string()),
        ConfigCommand::Show => {
            let payload = serde_json::json!({
                "config_path": config.paths.config_file,
                "data": config.data,
            });
            write_line(&serde_json::to_string_pretty(&payload).map_err(|err| {
                AppError::with_detail("E_OUTPUT", "failed to serialize config", err.to_string())
            })?)
        }
    }
}

fn auth_login(config: &ConfigBundle, host: &HostContext, args: &AuthLoginArgs) -> AppResult<()> {
    let token = if args.token_stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).map_err(|err| {
            AppError::with_detail(
                "E_AUTH_INVALID",
                "failed to read token from stdin",
                err.to_string(),
            )
        })?;
        buffer.trim().to_string()
    } else {
        prompt_for_token(host)?
    };

    if token.trim().is_empty() {
        return Err(AppError::new("E_AUTH_INVALID", "token must not be empty"));
    }

    let client = GitHubClient::new(host.api_base.clone(), token.clone())?;
    let identity = client.fetch_authenticated_user()?;
    let source = save_token(host, &token, config)?;
    if matches!(source, CredentialSource::InsecureFile) {
        eprintln!(
            "warning: secure storage unavailable, token stored in an insecure file by explicit opt-in"
        );
    }
    write_line(&format!(
        "logged in as {} for {}",
        identity.login, host.web_host
    ))
}

fn auth_status(config: &ConfigBundle, host: &HostContext) -> AppResult<()> {
    let env_source = env_credential_source(host);

    if let Some(source) = env_source {
        match saved_credential_source(host, config) {
            Ok(Some(saved)) => write_line(&format!(
                "environment override active for {} via {:?} (saved token also present via {:?})",
                host.web_host, source, saved
            )),
            Ok(None) => write_line(&format!(
                "no saved token for {} (environment override active: {:?})",
                host.web_host, source
            )),
            Err(error) => write_line(&format!(
                "environment override active for {} via {:?} (saved credential state unavailable: {})",
                host.web_host, source, error.message
            )),
        }
    } else if let Some(source) = saved_credential_source(host, config)? {
        write_line(&format!(
            "saved token present for {} via {:?}",
            host.web_host, source
        ))
    } else {
        write_line(&format!("no saved token for {}", host.web_host))
    }
}

fn auth_logout(config: &ConfigBundle, host: &HostContext) -> AppResult<()> {
    let deleted = delete_token(host, config)?;
    if deleted {
        write_line(&format!("logged out from {}", host.web_host))
    } else {
        write_line(&format!("no saved token for {}", host.web_host))
    }
}

fn resolve_host(cli: &Cli, config: &ConfigBundle) -> AppResult<HostContext> {
    let host = cli.host.as_deref().or(config.data.host.as_deref());
    normalize_host(host)
}

fn prompt_for_token(host: &HostContext) -> AppResult<String> {
    let interactive = io::stdin().is_terminal();
    if !interactive {
        return Err(AppError::new(
            "E_AUTH_INVALID",
            "auth login requires a TTY unless you use --token-stdin",
        ));
    }

    eprintln!("GitHub personal access token setup for {}", host.web_host);
    eprintln!("1. Open the personal access token settings page for this host.");
    eprintln!("2. Prefer a fine-grained token with read-only repository metadata access.");
    eprintln!("3. Create the token, copy it, and paste it below.");
    eprint!("Paste token: ");
    io::stderr().flush().ok();

    let mut token = String::new();
    io::stdin().read_line(&mut token).map_err(|err| {
        AppError::with_detail("E_AUTH_INVALID", "failed to read token", err.to_string())
    })?;
    Ok(token.trim().to_string())
}

fn parse_owner_repo(value: &str) -> AppResult<(String, String)> {
    let mut parts = value.split('/');
    let owner = parts.next().unwrap_or_default().trim();
    let repo = parts.next().unwrap_or_default().trim();
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return Err(AppError::new(
            "E_FLAG_CONFLICT",
            "repository must be in owner/repo form",
        ));
    }
    Ok((owner.to_string(), repo.to_string()))
}

fn progress_enabled(mode: ProgressMode) -> bool {
    match mode {
        ProgressMode::On => true,
        ProgressMode::Off => false,
        ProgressMode::Auto => io::stderr().is_terminal(),
    }
}

fn sort_native_results(repos: &mut [Repository], sort: SearchSort) {
    match sort {
        SearchSort::BestMatch => {}
        SearchSort::Stars => repos.sort_by(|left, right| {
            right
                .stargazers_count
                .cmp(&left.stargazers_count)
                .then_with(|| right.forks_count.cmp(&left.forks_count))
                .then_with(|| left.full_name.cmp(&right.full_name))
        }),
        SearchSort::Updated => repos.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.pushed_at.cmp(&left.pushed_at))
                .then_with(|| right.stargazers_count.cmp(&left.stargazers_count))
                .then_with(|| left.full_name.cmp(&right.full_name))
        }),
    }
}

fn discovery_search(
    client: &GitHubClient,
    plan: &crate::query::SearchPlan,
    show_progress: bool,
) -> AppResult<Vec<Repository>> {
    use std::collections::HashSet;

    let target = discovery_target(plan.depth, plan.limit);
    let mut pool = Vec::new();
    let mut seen = HashSet::new();

    let collect = |pool: &mut Vec<Repository>, seen: &mut HashSet<String>, repos: Vec<Repository>| {
        for repo in repos {
            if seen.contains(&repo.full_name) {
                continue;
            }
            seen.insert(repo.full_name.clone());
            pool.push(repo);
        }
    };

    progress(
        show_progress,
        format!("collecting seed candidates target={target}"),
    );
    let seed = client.search_repositories(
        &plan.compiled_query,
        plan.sort.as_github_value(),
        target.min(100),
        1,
    )?;
    collect(&mut pool, &mut seen, seed.items);
    if pool.len() >= target {
        return Ok(pool);
    }

    if matches!(
        plan.depth,
        crate::model::DiscoveryDepth::Balanced | crate::model::DiscoveryDepth::Deep
    ) {
        progress(show_progress, "collecting updated shard");
        let updated = client.search_repositories(
            &plan.compiled_query,
            Some("updated"),
            target.min(100),
            1,
        )?;
        collect(&mut pool, &mut seen, updated.items);
        if pool.len() >= target {
            return Ok(pool);
        }

        progress(show_progress, "collecting recent pushed shard");
        let recent_query = format!(
            "{} pushed:>={}",
            plan.compiled_query,
            (Utc::now() - Duration::days(30)).format("%Y-%m-%d")
        );
        let recent =
            client.search_repositories(&recent_query, Some("updated"), target.min(100), 1)?;
        collect(&mut pool, &mut seen, recent.items);
        if pool.len() >= target {
            return Ok(pool);
        }
    }

    if matches!(plan.depth, crate::model::DiscoveryDepth::Deep) {
        let pushed_buckets = [
            (Duration::days(180), Duration::days(30)),
            (Duration::days(365), Duration::days(180)),
        ];
        for (older_than, newer_than) in pushed_buckets {
            if pool.len() >= target {
                break;
            }
            progress(show_progress, "collecting older pushed bucket shard");
            let older_query = format!(
                "{} pushed:{}..{}",
                plan.compiled_query,
                (Utc::now() - older_than).format("%Y-%m-%d"),
                (Utc::now() - newer_than).format("%Y-%m-%d")
            );
            let older =
                client.search_repositories(&older_query, Some("updated"), target.min(100), 1)?;
            collect(&mut pool, &mut seen, older.items);
        }

        if !compiled_query_has_qualifier(&plan.compiled_query, "stars") {
            let star_buckets = ["50..499", "500..4999", ">=5000"];
            for bucket in star_buckets {
                if pool.len() >= target {
                    break;
                }
                progress(show_progress, "collecting star bucket shard");
                let star_query = format!("{} stars:{}", plan.compiled_query, bucket);
                let stars =
                    client.search_repositories(&star_query, Some("stars"), target.min(100), 1)?;
                collect(&mut pool, &mut seen, stars.items);
            }
        }
    }

    Ok(pool)
}

fn enrich_metadata(
    client: &GitHubClient,
    repos: &mut [Repository],
    concurrency: usize,
    show_progress: bool,
) -> AppResult<()> {
    progress(show_progress, "enriching metadata");
    if concurrency == 1 {
        for repo in repos {
            let (owner, name) = parse_owner_repo(&repo.full_name)?;
            let detail = client.repository(&owner, &name)?;
            repo.license = detail.license;
            repo.topics = detail.topics;
            repo.open_issues_count = detail.open_issues_count;
            repo.is_template = detail.is_template;
            repo.contributor_count = client.contributor_count(&owner, &name)?;
            repo.latest_release = client.latest_release(&owner, &name)?;
        }
        return Ok(());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .map_err(|err| {
            AppError::with_detail(
                "E_HTTP",
                "failed to initialize worker pool",
                err.to_string(),
            )
        })?;

    let details = pool.install(|| {
        repos
            .par_iter()
            .map(|repo| {
                let (owner, name) = parse_owner_repo(&repo.full_name)?;
                let detail = client.repository(&owner, &name)?;
                let contributors = client.contributor_count(&owner, &name)?;
                let release = client.latest_release(&owner, &name)?;
                Ok::<_, AppError>((repo.full_name.clone(), detail, contributors, release))
            })
            .collect::<Vec<_>>()
    });

    for result in details {
        let (full_name, detail, contributors, release) = result?;
        if let Some(repo) = repos.iter_mut().find(|repo| repo.full_name == full_name) {
            repo.license = detail.license;
            repo.topics = detail.topics;
            repo.open_issues_count = detail.open_issues_count;
            repo.is_template = detail.is_template;
            repo.contributor_count = contributors;
            repo.latest_release = release;
        }
    }

    Ok(())
}

fn enrich_readme_window(
    client: &GitHubClient,
    repos: &mut [Repository],
    limit: usize,
    concurrency: usize,
) -> AppResult<()> {
    let window = repos.len().min(20).min((limit * 2).max(10));
    if window == 0 {
        return Ok(());
    }

    if concurrency == 1 {
        for repo in repos.iter_mut().take(window) {
            let (owner, name) = parse_owner_repo(&repo.full_name)?;
            repo.readme = client.readme(&owner, &name)?;
        }
        return Ok(());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .map_err(|err| {
            AppError::with_detail(
                "E_HTTP",
                "failed to initialize worker pool",
                err.to_string(),
            )
        })?;

    let updates = pool.install(|| {
        repos[..window]
            .par_iter()
            .map(|repo| {
                let (owner, name) = parse_owner_repo(&repo.full_name)?;
                let readme = client.readme(&owner, &name)?;
                Ok::<_, AppError>((repo.full_name.clone(), readme))
            })
            .collect::<Vec<_>>()
    });

    for result in updates {
        let (full_name, readme) = result?;
        if let Some(repo) = repos.iter_mut().find(|repo| repo.full_name == full_name) {
            repo.readme = readme;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sort_native_results;
    use crate::model::{Owner, Repository, SearchSort};
    use crate::query::compiled_query_has_qualifier;
    use chrono::{TimeZone, Utc};

    fn repo(name: &str, stars: u64, updated_day: u32) -> Repository {
        Repository {
            name: name.to_string(),
            full_name: format!("example/{name}"),
            html_url: format!("https://example.test/{name}"),
            description: Some("fixture".to_string()),
            stargazers_count: stars,
            forks_count: stars / 10,
            language: Some("Rust".to_string()),
            topics: vec![],
            license: None,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 4, updated_day, 0, 0, 0).unwrap(),
            pushed_at: Utc.with_ymd_and_hms(2026, 4, updated_day, 0, 0, 0).unwrap(),
            archived: false,
            is_template: false,
            fork: false,
            open_issues_count: Some(0),
            owner: Owner {
                login: "example".to_string(),
            },
            readme: None,
            latest_release: None,
            contributor_count: None,
            explain: None,
        }
    }

    #[test]
    fn native_star_sort_orders_descending() {
        let mut repos = vec![repo("small", 10, 10), repo("large", 100, 5)];
        sort_native_results(&mut repos, SearchSort::Stars);
        assert_eq!(repos[0].name, "large");
    }

    #[test]
    fn native_updated_sort_orders_descending() {
        let mut repos = vec![repo("older", 10, 5), repo("newer", 10, 10)];
        sort_native_results(&mut repos, SearchSort::Updated);
        assert_eq!(repos[0].name, "newer");
    }

    #[test]
    fn detects_star_qualifier_case_insensitively() {
        assert!(compiled_query_has_qualifier("rust cli Stars:>10", "stars"));
        assert!(!compiled_query_has_qualifier("rust cli forks:>10", "stars"));
    }
}
