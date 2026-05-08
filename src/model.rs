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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SearchPatternMode {
    #[default]
    Literal,
    Regex,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TreeEntryKind {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub kind: TreeEntryKind,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeOutput {
    pub host: String,
    pub repository: String,
    pub reference: String,
    pub truncated: bool,
    pub total_count: usize,
    pub items: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMatchLine {
    pub line: usize,
    pub text: String,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMatch {
    pub path: String,
    pub line: usize,
    pub lines: Vec<CodeMatchLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchOutput {
    pub host: String,
    pub repository: String,
    pub reference: String,
    pub pattern: String,
    pub mode: SearchPatternMode,
    pub searched_files: usize,
    pub skipped_files: usize,
    pub total_count: usize,
    pub items: Vec<CodeMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialSource {
    EnvHost,
    EnvGlobal,
    Keyring,
    InsecureFile,
}
