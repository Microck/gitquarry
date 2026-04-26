use crate::error::{AppError, AppResult};
use crate::model::{LicenseInfo, ReleaseSummary, Repository};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

const API_VERSION: &str = "2026-03-10";

/// Maximum retry attempts for rate-limited requests (initial + retries).
const MAX_RETRY_ATTEMPTS: usize = 3;

#[derive(Clone)]
pub struct GitHubClient {
    http: Client,
    api_base: String,
}

#[derive(Debug, Clone)]
pub struct SearchPage {
    pub total_count: usize,
    pub items: Vec<Repository>,
}

#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub login: String,
}

impl GitHubClient {
    pub fn new(api_base: impl Into<String>, token: impl Into<String>) -> AppResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static(API_VERSION),
        );
        headers.insert(USER_AGENT, HeaderValue::from_str(&format!("gitquarry/{}", env!("CARGO_PKG_VERSION"))).map_err(|err| {
            AppError::with_detail("E_HTTP", "invalid user-agent header", err.to_string())
        })?);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.into())).map_err(|err| {
                AppError::with_detail("E_HTTP", "invalid authorization header", err.to_string())
            })?,
        );

        let http = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|err| {
                AppError::with_detail("E_HTTP", "failed to build HTTP client", err.to_string())
            })?;

        Ok(Self {
            http,
            api_base: api_base.into(),
        })
    }

    /// Fetches the authenticated user identity by calling the GitHub /user endpoint.
    /// This validates the token and returns the user's login.
    pub fn fetch_authenticated_user(&self) -> AppResult<AuthIdentity> {
        let response = self.send_with_retry(|| self.http.get(format!("{}/user", self.api_base)))?;
        let user: AuthUser = parse_json(response)?;
        Ok(AuthIdentity { login: user.login })
    }

    pub fn search_repositories(
        &self,
        query: &str,
        sort: Option<&str>,
        per_page: usize,
        page: usize,
    ) -> AppResult<SearchPage> {
        let response = self.send_with_retry(|| {
            let mut request = self
                .http
                .get(format!("{}/search/repositories", self.api_base))
                .query(&[
                    ("q", query),
                    ("per_page", &per_page.to_string()),
                    ("page", &page.to_string()),
                ]);

            if let Some(sort) = sort {
                request = request.query(&[("sort", sort), ("order", "desc")]);
            }

            request
        })?;

        let payload: SearchResponse = parse_json(response)?;
        Ok(SearchPage {
            total_count: payload.total_count,
            items: payload.items.into_iter().map(Repository::from).collect(),
        })
    }

    pub fn repository(&self, owner: &str, repo: &str) -> AppResult<Repository> {
        let response = self.send_with_retry(|| {
            self.http
                .get(format!("{}/repos/{owner}/{repo}", self.api_base))
        })?;
        let payload: RepositoryResponse = parse_json(response)?;
        Ok(payload.into())
    }

    pub fn readme(&self, owner: &str, repo: &str) -> AppResult<Option<String>> {
        let response = self.send_with_retry(|| {
            self.http
                .get(format!("{}/repos/{owner}/{repo}/readme", self.api_base))
                .header(ACCEPT, "application/vnd.github.raw+json")
        });

        match response {
            Ok(response) => Ok(Some(response.text().map_err(|err| {
                AppError::with_detail("E_HTTP", "failed to read README body", err.to_string())
            })?)),
            Err(error) if error.code == "E_NOT_FOUND" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn latest_release(&self, owner: &str, repo: &str) -> AppResult<Option<ReleaseSummary>> {
        let response = self.send_with_retry(|| {
            self.http.get(format!(
                "{}/repos/{owner}/{repo}/releases/latest",
                self.api_base
            ))
        });
        match response {
            Ok(response) => {
                let payload: ReleaseResponse = parse_json(response)?;
                Ok(Some(payload.into()))
            }
            Err(error) if error.code == "E_NOT_FOUND" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn contributor_count(&self, owner: &str, repo: &str) -> AppResult<Option<u64>> {
        let response = self.send_with_retry(|| {
            self.http
                .get(format!(
                    "{}/repos/{owner}/{repo}/contributors",
                    self.api_base
                ))
                .query(&[("per_page", "1"), ("anon", "1")])
        });

        let response = match response {
            Ok(response) => response,
            Err(error) if error.code == "E_NOT_FOUND" => return Ok(None),
            Err(error) if is_non_fatal_contributor_count_error(&error) => return Ok(None),
            Err(error) => return Err(error),
        };

        if response.status() == StatusCode::NO_CONTENT {
            return Ok(Some(0));
        }

        let link = response
            .headers()
            .get("link")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        if let Some(link) = link
            && let Some(last_page) = parse_last_page(&link)
        {
            return Ok(Some(last_page as u64));
        }

        let contributors: Vec<serde_json::Value> = parse_json(response)?;
        Ok(Some(contributors.len() as u64))
    }

    fn send_with_retry<F>(&self, build: F) -> AppResult<Response>
    where
        F: Fn() -> reqwest::blocking::RequestBuilder,
    {
        let mut attempts = 0usize;
        loop {
            attempts += 1;
            let response = build().send().map_err(|err| {
                AppError::with_detail("E_HTTP", "request failed", err.to_string())
            })?;

            if response.status().is_success() {
                return Ok(response);
            }

            if response.status().as_u16() == 404 {
                return Err(AppError::new("E_NOT_FOUND", "resource not found"));
            }

            if (response.status().as_u16() == 403 || response.status().as_u16() == 429)
                && attempts < MAX_RETRY_ATTEMPTS
            {
                let wait = retry_delay(&response)
                    .unwrap_or_else(|| Duration::from_millis((attempts * 1_000) as u64 + jitter()));
                thread::sleep(wait);
                continue;
            }

            let status = response.status();
            let body = response.text().unwrap_or_default();
            if status.as_u16() == 401 {
                return Err(AppError::new("E_AUTH_INVALID", "token rejected by GitHub"));
            }

            return Err(AppError::new(
                "E_GITHUB_API",
                format!(
                    "GitHub API request failed with status {} {}",
                    status.as_u16(),
                    body.trim()
                ),
            ));
        }
    }
}

fn retry_delay(response: &Response) -> Option<Duration> {
    if let Some(value) = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        && let Ok(seconds) = value.parse::<u64>()
    {
        return Some(Duration::from_secs(seconds));
    }

    let reset = response
        .headers()
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(Duration::from_secs(reset.saturating_sub(now)))
}

fn jitter() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| (value.as_millis() % 250) as u64)
        .unwrap_or(0)
}

fn parse_json<T: for<'de> Deserialize<'de>>(response: Response) -> AppResult<T> {
    response.json::<T>().map_err(|err| {
        AppError::with_detail("E_HTTP", "failed to decode response body", err.to_string())
    })
}

fn parse_last_page(link: &str) -> Option<usize> {
    link.split(',').find_map(|segment| {
        let segment = segment.trim();
        if !segment.contains("rel=\"last\"") {
            return None;
        }
        let start = segment.find('<')? + 1;
        let end = segment[start..].find('>')? + start;
        let url = Url::parse(&segment[start..end]).ok()?;
        url.query_pairs().find_map(|(key, value)| {
            if key == "page" {
                value.parse::<usize>().ok()
            } else {
                None
            }
        })
    })
}

fn is_non_fatal_contributor_count_error(error: &AppError) -> bool {
    error.code == "E_HTTP"
        || (error.code == "E_GITHUB_API"
            && error.message.contains("too large to list contributors"))
}

#[cfg(test)]
mod tests {
    use super::{is_non_fatal_contributor_count_error, parse_last_page};
    use crate::error::AppError;

    #[test]
    fn parse_last_page_reads_page_query_not_per_page() {
        let link = concat!(
            "<https://api.github.com/repositories/724712/contributors?per_page=1&anon=1&page=2>; rel=\"next\", ",
            "<https://api.github.com/repositories/724712/contributors?per_page=1&anon=1&page=8364>; rel=\"last\""
        );

        assert_eq!(parse_last_page(link), Some(8364));
    }

    #[test]
    fn parse_last_page_returns_none_without_last_relation() {
        let link = "<https://api.github.com/repositories/724712/contributors?per_page=1&anon=1&page=2>; rel=\"next\"";

        assert_eq!(parse_last_page(link), None);
    }

    #[test]
    fn contributor_count_treats_large_repo_and_transport_failures_as_non_fatal() {
        assert!(is_non_fatal_contributor_count_error(&AppError::new(
            "E_HTTP",
            "request failed"
        )));
        assert!(is_non_fatal_contributor_count_error(&AppError::new(
            "E_GITHUB_API",
            "GitHub API request failed with status 403 {\"message\":\"The history or contributor list is too large to list contributors for this repository via the API.\"}"
        )));
        assert!(!is_non_fatal_contributor_count_error(&AppError::new(
            "E_AUTH_INVALID",
            "token rejected by GitHub"
        )));
    }
}

#[derive(Debug, Deserialize)]
struct AuthUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    total_count: usize,
    items: Vec<RepositoryResponse>,
}

#[derive(Debug, Deserialize)]
struct RepositoryResponse {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u64,
    forks_count: u64,
    language: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    license: Option<LicenseResponse>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pushed_at: DateTime<Utc>,
    archived: bool,
    #[serde(default)]
    is_template: bool,
    fork: bool,
    open_issues_count: Option<u64>,
    owner: OwnerResponse,
}

impl From<RepositoryResponse> for Repository {
    fn from(value: RepositoryResponse) -> Self {
        Self {
            name: value.name,
            full_name: value.full_name,
            html_url: value.html_url,
            description: value.description,
            stargazers_count: value.stargazers_count,
            forks_count: value.forks_count,
            language: value.language,
            topics: value.topics,
            license: value.license.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
            pushed_at: value.pushed_at,
            archived: value.archived,
            is_template: value.is_template,
            fork: value.fork,
            open_issues_count: value.open_issues_count,
            owner: value.owner.into(),
            readme: None,
            latest_release: None,
            contributor_count: None,
            explain: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OwnerResponse {
    login: String,
}

impl From<OwnerResponse> for crate::model::Owner {
    fn from(value: OwnerResponse) -> Self {
        Self { login: value.login }
    }
}

#[derive(Debug, Deserialize)]
struct LicenseResponse {
    key: Option<String>,
    name: Option<String>,
    spdx_id: Option<String>,
}

impl From<LicenseResponse> for LicenseInfo {
    fn from(value: LicenseResponse) -> Self {
        Self {
            key: value.key,
            name: value.name,
            spdx_id: value.spdx_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    name: Option<String>,
    published_at: Option<DateTime<Utc>>,
    html_url: String,
}

impl From<ReleaseResponse> for ReleaseSummary {
    fn from(value: ReleaseResponse) -> Self {
        Self {
            tag_name: value.tag_name,
            name: value.name,
            published_at: value.published_at,
            html_url: value.html_url,
        }
    }
}
