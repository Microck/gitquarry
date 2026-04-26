use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    Pretty,
    Json,
    Compact,
    Csv,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProgressMode {
    #[default]
    Auto,
    On,
    Off,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ColorPreference {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    #[default]
    Native,
    Discover,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RankMode {
    #[default]
    Native,
    Query,
    Activity,
    Quality,
    Blended,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SearchSort {
    #[default]
    BestMatch,
    Stars,
    Updated,
}

impl SearchSort {
    pub fn as_github_value(self) -> Option<&'static str> {
        match self {
            Self::BestMatch => None,
            Self::Stars => Some("stars"),
            Self::Updated => Some("updated"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DiscoveryDepth {
    Quick,
    #[default]
    Balanced,
    Deep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BoolFlag {
    True,
    False,
}

impl BoolFlag {
    pub fn as_bool(self) -> bool {
        matches!(self, Self::True)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ForkMode {
    False,
    True,
    Only,
}

impl ForkMode {
    pub fn qualifier(self) -> &'static str {
        match self {
            Self::False => "false",
            Self::True => "true",
            Self::Only => "only",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub key: Option<String>,
    pub name: Option<String>,
    pub spdx_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSummary {
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainScore {
    pub query: Option<f64>,
    pub activity: Option<f64>,
    pub quality: Option<f64>,
    pub blended: Option<f64>,
    pub weights: Option<ScoreWeights>,
    pub matched_surfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub query: f64,
    pub activity: f64,
    pub quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub license: Option<LicenseInfo>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub archived: bool,
    pub is_template: bool,
    pub fork: bool,
    pub open_issues_count: Option<u64>,
    pub owner: Owner,
    pub readme: Option<String>,
    pub latest_release: Option<ReleaseSummary>,
    pub contributor_count: Option<u64>,
    pub explain: Option<ExplainScore>,
}

/// Search output containing matched repositories.
///
/// Note: `total_count` semantics differ between retrieval modes:
/// - In Native mode: total count from GitHub API (total matching repos across all pages)
/// - In Discover mode: total count is the local pool size before post-filtering/truncation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOutput {
    pub host: String,
    pub mode: RetrievalMode,
    pub rank: RankMode,
    pub query: Option<String>,
    pub compiled_query: String,
    pub limit: usize,
    pub total_count: usize,
    pub items: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectOutput {
    pub host: String,
    pub repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialSource {
    EnvHost,
    EnvGlobal,
    Keyring,
    InsecureFile,
}
