use crate::error::{AppError, AppResult};
use crate::model::{ColorPreference, InspectOutput, OutputFormat, Repository, SearchOutput};
use anstream::eprintln;
use csv::Writer;
use std::io::{self, IsTerminal, Write};

const BLUE: &str = "\x1b[38;5;27m";
const RESET: &str = "\x1b[0m";

pub fn write_search(
    output: &SearchOutput,
    format: OutputFormat,
    color: ColorPreference,
) -> AppResult<()> {
    match format {
        OutputFormat::Pretty => print_pretty_search(output, color),
        OutputFormat::Json => print_json(output, true),
        OutputFormat::Compact => print_json(output, false),
        OutputFormat::Csv => print_csv(&output.items),
    }
}

pub fn write_inspect(
    output: &InspectOutput,
    format: OutputFormat,
    color: ColorPreference,
) -> AppResult<()> {
    match format {
        OutputFormat::Pretty => print_pretty_inspect(output, color),
        OutputFormat::Json => print_json(output, true),
        OutputFormat::Compact => print_json(output, false),
        OutputFormat::Csv => print_csv(std::slice::from_ref(&output.repository)),
    }
}

pub fn write_line(message: &str) -> AppResult<()> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(format!("{message}\n").as_bytes())
        .map_err(|err| AppError::with_detail("E_OUTPUT", "failed to write stdout", err.to_string()))
}

pub fn progress(enabled: bool, message: impl AsRef<str>) {
    if enabled {
        eprintln!("[gitquarry] {}", message.as_ref());
    }
}

fn print_pretty_search(output: &SearchOutput, color: ColorPreference) -> AppResult<()> {
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "{}{} results{}  mode={} rank={} host={}",
        accent(color),
        output.total_count,
        reset(color),
        serde_json::to_string(&output.mode)
            .unwrap_or_else(|_| "\"native\"".to_string())
            .trim_matches('"'),
        serde_json::to_string(&output.rank)
            .unwrap_or_else(|_| "\"native\"".to_string())
            .trim_matches('"'),
        output.host
    )
    .map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
    })?;

    for repo in &output.items {
        write_repo_block(&mut stdout, repo, color)?;
    }

    Ok(())
}

fn print_pretty_inspect(output: &InspectOutput, color: ColorPreference) -> AppResult<()> {
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "{}{}{}",
        accent(color),
        output.repository.full_name,
        reset(color)
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    write_repo_detail(&mut stdout, &output.repository)?;
    Ok(())
}

fn write_repo_block(
    stdout: &mut impl Write,
    repo: &Repository,
    color: ColorPreference,
) -> AppResult<()> {
    writeln!(
        stdout,
        "{}{}{}",
        accent(color),
        repo.full_name,
        reset(color)
    )
    .map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
    })?;
    writeln!(stdout, "  {}", repo.html_url).map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
    })?;
    if let Some(description) = &repo.description {
        writeln!(stdout, "  {description}").map_err(|err| {
            AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
        })?;
    }
    writeln!(
        stdout,
        "  stars={} forks={} language={} updated={}",
        repo.stargazers_count,
        repo.forks_count,
        repo.language.clone().unwrap_or_else(|| "-".to_string()),
        repo.updated_at.format("%Y-%m-%d")
    )
    .map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
    })?;
    if let Some(explain) = &repo.explain {
        writeln!(
            stdout,
            "  score query={:.3} activity={:.3} quality={:.3} blended={:.3}",
            explain.query.unwrap_or(0.0),
            explain.activity.unwrap_or(0.0),
            explain.quality.unwrap_or(0.0),
            explain.blended.unwrap_or(0.0)
        )
        .map_err(|err| {
            AppError::with_detail("E_OUTPUT", "failed to write pretty output", err.to_string())
        })?;
    }
    Ok(())
}

fn write_repo_detail(stdout: &mut impl Write, repo: &Repository) -> AppResult<()> {
    writeln!(stdout, "url: {}", repo.html_url).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(
        stdout,
        "description: {}",
        repo.description.clone().unwrap_or_else(|| "-".to_string())
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "stars: {}", repo.stargazers_count).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "forks: {}", repo.forks_count).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(
        stdout,
        "language: {}",
        repo.language.clone().unwrap_or_else(|| "-".to_string())
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(
        stdout,
        "topics: {}",
        if repo.topics.is_empty() {
            "-".to_string()
        } else {
            repo.topics.join(", ")
        }
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(
        stdout,
        "license: {}",
        repo.license
            .as_ref()
            .and_then(|license| license.spdx_id.clone().or_else(|| license.name.clone()))
            .unwrap_or_else(|| "-".to_string())
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "created: {}", repo.created_at.format("%Y-%m-%d")).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "updated: {}", repo.updated_at.format("%Y-%m-%d")).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "pushed: {}", repo.pushed_at.format("%Y-%m-%d")).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "archived: {}", repo.archived).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "template: {}", repo.is_template).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(stdout, "fork: {}", repo.fork).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    writeln!(
        stdout,
        "open_issues: {}",
        repo.open_issues_count.unwrap_or(0)
    )
    .map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write inspect output",
            err.to_string(),
        )
    })?;
    if let Some(release) = &repo.latest_release {
        writeln!(
            stdout,
            "latest_release: {} ({})",
            release.tag_name,
            release
                .published_at
                .map(|value| value.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string())
        )
        .map_err(|err| {
            AppError::with_detail(
                "E_OUTPUT",
                "failed to write inspect output",
                err.to_string(),
            )
        })?;
    }
    if let Some(readme) = &repo.readme {
        writeln!(stdout, "\nREADME\n------\n{}", readme.trim()).map_err(|err| {
            AppError::with_detail(
                "E_OUTPUT",
                "failed to write inspect output",
                err.to_string(),
            )
        })?;
    }
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T, pretty: bool) -> AppResult<()> {
    let raw = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to serialize output", err.to_string())
    })?;
    write_line(&raw)
}

fn print_csv(repos: &[Repository]) -> AppResult<()> {
    let raw = render_csv(repos)?;
    write_line(raw.trim_end())
}

fn render_csv(repos: &[Repository]) -> AppResult<String> {
    let mut writer = Writer::from_writer(Vec::new());
    writer
        .write_record([
            "full_name",
            "html_url",
            "description",
            "stars",
            "forks",
            "language",
            "topics",
            "license",
            "created_at",
            "updated_at",
            "pushed_at",
            "archived",
            "template",
            "fork",
            "open_issues_count",
            "contributor_count",
            "query_score",
            "activity_score",
            "quality_score",
            "blended_score",
        ])
        .map_err(|err| {
            AppError::with_detail("E_OUTPUT", "failed to write CSV header", err.to_string())
        })?;

    for repo in repos {
        let explain = repo.explain.as_ref();
        writer
            .write_record([
                repo.full_name.clone(),
                repo.html_url.clone(),
                repo.description.clone().unwrap_or_default(),
                repo.stargazers_count.to_string(),
                repo.forks_count.to_string(),
                repo.language.clone().unwrap_or_default(),
                repo.topics.join("|"),
                repo.license
                    .as_ref()
                    .and_then(|license| license.spdx_id.clone().or_else(|| license.name.clone()))
                    .unwrap_or_default(),
                repo.created_at.to_rfc3339(),
                repo.updated_at.to_rfc3339(),
                repo.pushed_at.to_rfc3339(),
                repo.archived.to_string(),
                repo.is_template.to_string(),
                repo.fork.to_string(),
                repo.open_issues_count.unwrap_or(0).to_string(),
                repo.contributor_count.unwrap_or(0).to_string(),
                explain
                    .and_then(|value| value.query)
                    .unwrap_or(0.0)
                    .to_string(),
                explain
                    .and_then(|value| value.activity)
                    .unwrap_or(0.0)
                    .to_string(),
                explain
                    .and_then(|value| value.quality)
                    .unwrap_or(0.0)
                    .to_string(),
                explain
                    .and_then(|value| value.blended)
                    .unwrap_or(0.0)
                    .to_string(),
            ])
            .map_err(|err| {
                AppError::with_detail("E_OUTPUT", "failed to write CSV row", err.to_string())
            })?;
    }

    let bytes = writer.into_inner().map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to flush CSV output", err.to_string())
    })?;
    let raw = String::from_utf8(bytes).map_err(|err| {
        AppError::with_detail("E_OUTPUT", "failed to decode CSV output", err.to_string())
    })?;
    Ok(raw)
}

fn accent(color: ColorPreference) -> &'static str {
    if use_color(color) { BLUE } else { "" }
}

fn reset(color: ColorPreference) -> &'static str {
    if use_color(color) { RESET } else { "" }
}

fn use_color(color: ColorPreference) -> bool {
    match color {
        ColorPreference::Always => true,
        ColorPreference::Never => false,
        ColorPreference::Auto => io::stdout().is_terminal(),
    }
}

#[cfg(test)]
mod tests {
    use super::{render_csv, write_repo_block, write_repo_detail};
    use crate::model::{
        ColorPreference, ExplainScore, LicenseInfo, Owner, ReleaseSummary, Repository, ScoreWeights,
    };
    use chrono::{TimeZone, Utc};

    fn repo() -> Repository {
        Repository {
            name: "rocket".to_string(),
            full_name: "example/rocket".to_string(),
            html_url: "https://example.com/rocket".to_string(),
            description: Some("CLI, with \"quotes\"".to_string()),
            stargazers_count: 420,
            forks_count: 32,
            language: Some("Rust".to_string()),
            topics: vec!["cli".to_string(), "search".to_string()],
            license: Some(LicenseInfo {
                key: Some("mit".to_string()),
                name: Some("MIT License".to_string()),
                spdx_id: Some("MIT".to_string()),
            }),
            created_at: Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 4, 20, 0, 0, 0).unwrap(),
            pushed_at: Utc.with_ymd_and_hms(2026, 4, 19, 0, 0, 0).unwrap(),
            archived: false,
            is_template: false,
            fork: false,
            open_issues_count: Some(4),
            owner: Owner {
                login: "example".to_string(),
            },
            readme: Some("# Rocket\n\nREADME body.\n".to_string()),
            latest_release: Some(ReleaseSummary {
                tag_name: "v1.2.3".to_string(),
                name: Some("v1.2.3".to_string()),
                published_at: Some(Utc.with_ymd_and_hms(2026, 4, 18, 0, 0, 0).unwrap()),
                html_url: "https://example.com/rocket/releases/v1.2.3".to_string(),
            }),
            contributor_count: Some(3),
            explain: Some(ExplainScore {
                query: Some(0.9),
                activity: Some(0.8),
                quality: Some(0.7),
                blended: Some(0.85),
                weights: Some(ScoreWeights {
                    query: 1.0,
                    activity: 1.0,
                    quality: 1.0,
                }),
                matched_surfaces: vec!["name".to_string(), "readme".to_string()],
            }),
        }
    }

    #[test]
    fn render_csv_escapes_commas_and_quotes() {
        let raw = render_csv(&[repo()]).unwrap();
        let mut lines = raw.lines();
        let header = lines.next().unwrap();
        let row = lines.next().unwrap();

        assert!(header.starts_with("full_name,html_url,description"));
        assert!(row.contains("\"CLI, with \"\"quotes\"\"\""));
        assert!(row.contains("cli|search"));
        assert!(row.contains(",0.9,0.8,0.7,0.85"));
    }

    #[test]
    fn write_repo_block_includes_scores_without_color_when_disabled() {
        let mut output = Vec::new();
        write_repo_block(&mut output, &repo(), ColorPreference::Never).unwrap();
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("example/rocket"));
        assert!(rendered.contains("stars=420 forks=32 language=Rust updated=2026-04-20"));
        assert!(rendered.contains("score query=0.900 activity=0.800 quality=0.700 blended=0.850"));
        assert!(!rendered.contains("\u{1b}["));
    }

    #[test]
    fn write_repo_detail_includes_release_and_trimmed_readme() {
        let mut output = Vec::new();
        write_repo_detail(&mut output, &repo()).unwrap();
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("latest_release: v1.2.3 (2026-04-18)"));
        assert!(rendered.contains("README\n------\n# Rocket\n\nREADME body."));
        assert!(rendered.contains("license: MIT"));
    }
}
