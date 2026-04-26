use crate::model::{ExplainScore, RankMode, Repository, ScoreWeights};
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone)]
struct ComputedScores {
    query: f64,
    activity: f64,
    quality: f64,
    blended: f64,
    matched_surfaces: Vec<String>,
}

pub fn rerank(
    repos: &mut [Repository],
    rank: RankMode,
    query: Option<&str>,
    weights: &ScoreWeights,
    explain: bool,
    now: DateTime<Utc>,
) {
    let needles = tokenize(query.unwrap_or(""));

    let scores_by_repo = repos
        .iter()
        .map(|repo| {
            let query_score = score_query(repo, &needles);
            let activity_score = score_activity(repo, now);
            let quality_score = score_quality(repo);
            let blended_score = if weights.query + weights.activity + weights.quality > 0.0 {
                ((weights.query * query_score)
                    + (weights.activity * activity_score)
                    + (weights.quality * quality_score))
                    / (weights.query + weights.activity + weights.quality)
            } else {
                0.0
            };

            (
                repo.full_name.clone(),
                ComputedScores {
                    query: query_score,
                    activity: activity_score,
                    quality: quality_score,
                    blended: blended_score,
                    matched_surfaces: matched_surfaces(repo, &needles),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    repos.sort_by(|left, right| compare(left, right, rank, &scores_by_repo));

    for repo in repos.iter_mut() {
        if explain {
            let scores = scores_by_repo
                .get(&repo.full_name)
                .expect("missing score entry");
            repo.explain = Some(ExplainScore {
                query: Some(scores.query),
                activity: Some(scores.activity),
                quality: Some(scores.quality),
                blended: Some(scores.blended),
                weights: Some(weights.clone()),
                matched_surfaces: scores.matched_surfaces.clone(),
            });
        } else {
            repo.explain = None;
        }
    }
}

fn compare(
    left: &Repository,
    right: &Repository,
    rank: RankMode,
    scores_by_repo: &HashMap<String, ComputedScores>,
) -> Ordering {
    // RankMode::Native is handled earlier in the call chain (app.rs ensures rerank is only called
    // when rank != RankMode::Native). This function should never receive RankMode::Native.
    debug_assert!(!matches!(rank, RankMode::Native));

    let score = |repo: &Repository| {
        scores_by_repo
            .get(&repo.full_name)
            .expect("missing score entry")
    };
    let left_score = match rank {
        RankMode::Query => score(left).query,
        RankMode::Activity => score(left).activity,
        RankMode::Quality => score(left).quality,
        RankMode::Blended => score(left).blended,
        RankMode::Native => unreachable!("RankMode::Native should never reach compare()"),
    };
    let right_score = match rank {
        RankMode::Query => score(right).query,
        RankMode::Activity => score(right).activity,
        RankMode::Quality => score(right).quality,
        RankMode::Blended => score(right).blended,
        RankMode::Native => unreachable!("RankMode::Native should never reach compare()"),
    };

    right_score
        .partial_cmp(&left_score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| right.stargazers_count.cmp(&left.stargazers_count))
        .then_with(|| right.forks_count.cmp(&left.forks_count))
}

fn score_query(repo: &Repository, needles: &[String]) -> f64 {
    if needles.is_empty() {
        return 0.0;
    }

    let name = repo.name.to_ascii_lowercase();
    let description = repo
        .description
        .clone()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let topics = repo.topics.join(" ").to_ascii_lowercase();
    let readme = repo.readme.clone().unwrap_or_default().to_ascii_lowercase();

    let mut score = 0.0;
    for needle in needles {
        if name.contains(needle) {
            score += 3.0;
        }
        if description.contains(needle) {
            score += 2.0;
        }
        if topics.contains(needle) {
            score += 1.5;
        }
        if !readme.is_empty() && readme.contains(needle) {
            score += 1.0;
        }
    }

    let max_score = needles.len() as f64 * 7.5;
    (score / max_score).clamp(0.0, 1.0)
}

fn score_activity(repo: &Repository, now: DateTime<Utc>) -> f64 {
    let days_since_push = (now - repo.pushed_at).num_days().max(0) as f64;
    let days_since_update = (now - repo.updated_at).num_days().max(0) as f64;
    let push_score = recency(days_since_push, 180.0);
    let update_score = recency(days_since_update, 365.0);
    let release_score = repo
        .latest_release
        .as_ref()
        .and_then(|release| release.published_at)
        .map(|published_at| recency((now - published_at).num_days().max(0) as f64, 365.0))
        .unwrap_or(0.0);
    let archived_penalty = if repo.archived { 0.2 } else { 1.0 };
    ((push_score * 0.5) + (update_score * 0.3) + (release_score * 0.2)) * archived_penalty
}

fn score_quality(repo: &Repository) -> f64 {
    let stars = log_norm(repo.stargazers_count as f64, 50_000.0);
    let forks = log_norm(repo.forks_count as f64, 10_000.0);
    let contributors = log_norm(repo.contributor_count.unwrap_or(0) as f64, 500.0);
    let license = if repo.license.is_some() { 1.0 } else { 0.0 };
    let readme = if repo
        .readme
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        1.0
    } else {
        0.0
    };
    let template_penalty = if repo.is_template { 0.9 } else { 1.0 };
    (((stars * 0.4) + (forks * 0.25) + (contributors * 0.2) + (license * 0.1) + (readme * 0.05))
        * template_penalty)
        .clamp(0.0, 1.0)
}

fn recency(days: f64, half_life_days: f64) -> f64 {
    (0.5f64).powf(days / half_life_days)
}

fn log_norm(value: f64, scale: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    (value.ln_1p() / scale.ln_1p()).clamp(0.0, 1.0)
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|value| value.trim_matches(|ch: char| !ch.is_alphanumeric()))
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .collect()
}

fn matched_surfaces(repo: &Repository, needles: &[String]) -> Vec<String> {
    let mut surfaces = Vec::new();
    let contains_any = |haystack: &str| needles.iter().any(|needle| haystack.contains(needle));
    if contains_any(&repo.name.to_ascii_lowercase()) {
        surfaces.push("name".to_string());
    }
    if contains_any(
        &repo
            .description
            .clone()
            .unwrap_or_default()
            .to_ascii_lowercase(),
    ) {
        surfaces.push("description".to_string());
    }
    if contains_any(&repo.topics.join(" ").to_ascii_lowercase()) {
        surfaces.push("topics".to_string());
    }
    if contains_any(&repo.readme.clone().unwrap_or_default().to_ascii_lowercase()) {
        surfaces.push("readme".to_string());
    }
    surfaces
}

#[cfg(test)]
mod tests {
    use super::rerank;
    use crate::model::{LicenseInfo, Owner, RankMode, ReleaseSummary, Repository, ScoreWeights};
    use chrono::{TimeZone, Utc};

    fn repo(name: &str, stars: u64, forks: u64, description: &str) -> Repository {
        Repository {
            name: name.to_string(),
            full_name: format!("owner/{name}"),
            html_url: format!("https://example.com/{name}"),
            description: Some(description.to_string()),
            stargazers_count: stars,
            forks_count: forks,
            language: Some("Rust".to_string()),
            topics: vec!["cli".to_string()],
            license: None,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            pushed_at: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            archived: false,
            is_template: false,
            fork: false,
            open_issues_count: Some(0),
            owner: Owner {
                login: "owner".to_string(),
            },
            readme: Some("Rust CLI toolkit".to_string()),
            latest_release: None,
            contributor_count: Some(20),
            explain: None,
        }
    }

    fn recent_release() -> ReleaseSummary {
        ReleaseSummary {
            tag_name: "v1.0.0".to_string(),
            name: Some("v1.0.0".to_string()),
            published_at: Some(Utc.with_ymd_and_hms(2026, 4, 20, 0, 0, 0).unwrap()),
            html_url: "https://example.com/release".to_string(),
        }
    }

    #[test]
    fn query_rank_prefers_matching_name() {
        let mut repos = vec![
            repo("alpha", 100, 10, "search helper"),
            repo("rust-toolkit", 50, 5, "tooling"),
        ];
        rerank(
            &mut repos,
            RankMode::Query,
            Some("rust"),
            &ScoreWeights {
                query: 1.0,
                activity: 1.0,
                quality: 1.0,
            },
            true,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );
        assert_eq!(repos[0].name, "rust-toolkit");
    }

    #[test]
    fn rank_does_not_expose_explain_without_flag() {
        let mut repos = vec![repo("rust-toolkit", 50, 5, "tooling")];
        rerank(
            &mut repos,
            RankMode::Query,
            Some("rust"),
            &ScoreWeights {
                query: 1.0,
                activity: 1.0,
                quality: 1.0,
            },
            false,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );
        assert!(repos[0].explain.is_none());
    }

    #[test]
    fn activity_rank_prefers_recent_and_non_archived_repositories() {
        let mut fresh = repo("fresh", 10, 5, "tooling");
        fresh.updated_at = Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap();
        fresh.pushed_at = Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap();
        fresh.latest_release = Some(recent_release());

        let mut stale = repo("stale", 10, 5, "tooling");
        stale.updated_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        stale.pushed_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        stale.archived = true;

        let mut repos = vec![stale, fresh];
        rerank(
            &mut repos,
            RankMode::Activity,
            Some("tooling"),
            &ScoreWeights {
                query: 0.0,
                activity: 1.0,
                quality: 0.0,
            },
            true,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );

        assert_eq!(repos[0].name, "fresh");
        let explain = repos[0].explain.as_ref().unwrap();
        assert!(explain.activity.unwrap() > explain.query.unwrap());
    }

    #[test]
    fn quality_rank_prefers_stronger_repository_signals() {
        let mut strong = repo("strong", 5_000, 400, "tooling");
        strong.license = Some(LicenseInfo {
            key: Some("mit".to_string()),
            name: Some("MIT License".to_string()),
            spdx_id: Some("MIT".to_string()),
        });
        strong.contributor_count = Some(150);
        strong.readme = Some("documented tooling".to_string());

        let mut weak = repo("weak", 10, 1, "tooling");
        weak.contributor_count = Some(1);
        weak.readme = Some(String::new());

        let mut repos = vec![weak, strong];
        rerank(
            &mut repos,
            RankMode::Quality,
            Some("tooling"),
            &ScoreWeights {
                query: 0.0,
                activity: 0.0,
                quality: 1.0,
            },
            true,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );

        assert_eq!(repos[0].name, "strong");
        assert!(repos[0].explain.as_ref().unwrap().quality.unwrap() > 0.5);
    }

    #[test]
    fn blended_rank_with_zero_weight_component_ignores_that_component() {
        let mut query_match = repo("query-match", 10, 5, "rust automation");
        query_match.updated_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        query_match.pushed_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let mut active = repo("active", 10, 5, "generic tooling");
        active.updated_at = Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap();
        active.pushed_at = Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap();
        active.latest_release = Some(recent_release());

        let mut repos = vec![active, query_match];
        rerank(
            &mut repos,
            RankMode::Blended,
            Some("rust"),
            &ScoreWeights {
                query: 1.0,
                activity: 0.0,
                quality: 0.0,
            },
            true,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );

        assert_eq!(repos[0].name, "query-match");
        let explain = repos[0].explain.as_ref().unwrap();
        assert_eq!(explain.weights.as_ref().unwrap().activity, 0.0);
        assert!(
            explain
                .matched_surfaces
                .contains(&"description".to_string())
        );
    }

    #[test]
    fn zero_total_weights_fall_back_to_star_tiebreaker() {
        let mut low_star = repo("low-star", 10, 5, "tooling");
        let mut high_star = repo("high-star", 50, 5, "tooling");
        low_star.description = Some("neutral".to_string());
        high_star.description = Some("neutral".to_string());

        let mut repos = vec![low_star, high_star];
        rerank(
            &mut repos,
            RankMode::Blended,
            Some("missing"),
            &ScoreWeights {
                query: 0.0,
                activity: 0.0,
                quality: 0.0,
            },
            false,
            Utc.with_ymd_and_hms(2026, 4, 21, 0, 0, 0).unwrap(),
        );

        assert_eq!(repos[0].name, "high-star");
    }
}
