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
    write_line(raw.trim_end())
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
