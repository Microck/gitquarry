use crate::model::{
    BoolFlag, DiscoveryDepth, ForkMode, OutputFormat, ProgressMode, RankMode, RetrievalMode,
    SearchSort,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

impl CompletionShell {
    pub fn to_clap_shell(self) -> Shell {
        match self {
            Self::Bash => Shell::Bash,
            Self::Zsh => Shell::Zsh,
            Self::Fish => Shell::Fish,
            Self::Powershell => Shell::PowerShell,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "gitquarry",
    author,
    version,
    about = "Search public GitHub repositories with explicit discovery controls."
)]
pub struct Cli {
    /// GitHub.com hostname or a full custom GitHub host URL.
    #[arg(long, global = true)]
    pub host: Option<String>,

    /// Generate shell completion for the selected shell and print it to stdout.
    #[arg(long = "generate-completion", value_enum)]
    pub generate_completion: Option<CompletionShell>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search public repositories.
    Search(Box<SearchArgs>),
    /// Inspect one explicit owner/repo.
    Inspect(InspectArgs),
    /// Manage host-scoped personal access tokens.
    Auth(AuthArgs),
    /// Show config path or the effective config payload.
    Config(ConfigArgs),
    /// Print the current gitquarry version.
    Version,
}

#[derive(Debug, Args, Clone)]
pub struct SearchArgs {
    /// Free-text query. Required unless discover mode is used with only structured filters.
    pub query: Option<String>,

    /// Retrieval mode. Omit for native GitHub-like search.
    #[arg(long, value_enum)]
    pub mode: Option<RetrievalMode>,

    /// Ranking mode. Non-native ranks require --mode discover.
    #[arg(long, value_enum)]
    pub rank: Option<RankMode>,

    /// Native GitHub-like sort order.
    #[arg(long, value_enum, default_value_t = SearchSort::BestMatch)]
    pub sort: SearchSort,

    /// Discovery depth. Requires --mode discover.
    #[arg(long, value_enum)]
    pub depth: Option<DiscoveryDepth>,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Maximum number of repositories to print.
    #[arg(long)]
    pub limit: Option<usize>,

    /// Restrict search to one user.
    #[arg(long)]
    pub user: Option<String>,

    /// Restrict search to one organization.
    #[arg(long)]
    pub org: Option<String>,

    /// Filter archived repositories.
    #[arg(long, value_enum)]
    pub archived: Option<BoolFlag>,

    /// Filter template repositories.
    #[arg(long, value_enum)]
    pub template: Option<BoolFlag>,

    /// Filter fork state.
    #[arg(long, value_enum)]
    pub fork: Option<ForkMode>,

    /// Require one language. Repeat for AND semantics.
    #[arg(long)]
    pub language: Vec<String>,

    /// Require one topic. Repeat for AND semantics.
    #[arg(long)]
    pub topic: Vec<String>,

    /// Require one license. Repeat for AND semantics.
    #[arg(long)]
    pub license: Vec<String>,

    /// Minimum stars.
    #[arg(long)]
    pub min_stars: Option<u64>,

    /// Maximum stars.
    #[arg(long)]
    pub max_stars: Option<u64>,

    /// Minimum forks.
    #[arg(long)]
    pub min_forks: Option<u64>,

    /// Maximum forks.
    #[arg(long)]
    pub max_forks: Option<u64>,

    /// Minimum repository size in KB.
    #[arg(long)]
    pub min_size: Option<u64>,

    /// Maximum repository size in KB.
    #[arg(long)]
    pub max_size: Option<u64>,

    /// Created-on-or-after date in YYYY-MM-DD.
    #[arg(long)]
    pub created_after: Option<String>,

    /// Created-on-or-before date in YYYY-MM-DD.
    #[arg(long)]
    pub created_before: Option<String>,

    /// Updated-on-or-after date in YYYY-MM-DD.
    #[arg(long)]
    pub updated_after: Option<String>,

    /// Updated-on-or-before date in YYYY-MM-DD.
    #[arg(long)]
    pub updated_before: Option<String>,

    /// Pushed-on-or-after date in YYYY-MM-DD.
    #[arg(long)]
    pub pushed_after: Option<String>,

    /// Pushed-on-or-before date in YYYY-MM-DD.
    #[arg(long)]
    pub pushed_before: Option<String>,

    /// Require created recency like 30d, 12h, or 1y.
    #[arg(long)]
    pub created_within: Option<String>,

    /// Require updated recency like 30d, 12h, or 1y.
    #[arg(long)]
    pub updated_within: Option<String>,

    /// Require push recency like 30d, 12h, or 1y.
    #[arg(long)]
    pub pushed_within: Option<String>,

    /// Enrich the top candidate window with README content.
    #[arg(long, default_value_t = false)]
    pub readme: bool,

    /// Show ranking reasons for enhanced search.
    #[arg(long, default_value_t = false)]
    pub explain: bool,

    /// Blended query weight in the range 0.0..=3.0.
    #[arg(long)]
    pub weight_query: Option<f64>,

    /// Blended activity weight in the range 0.0..=3.0.
    #[arg(long)]
    pub weight_activity: Option<f64>,

    /// Blended quality weight in the range 0.0..=3.0.
    #[arg(long)]
    pub weight_quality: Option<f64>,

    /// Worker count for discover-mode enrichment.
    #[arg(long)]
    pub concurrency: Option<usize>,

    /// Progress output mode for stderr.
    #[arg(long, value_enum)]
    pub progress: Option<ProgressMode>,
}

#[derive(Debug, Args, Clone)]
pub struct InspectArgs {
    /// Explicit repository identifier in owner/repo form.
    pub repository: String,

    /// Include the repository README in the output.
    #[arg(long, default_value_t = false)]
    pub readme: bool,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Progress output mode for stderr.
    #[arg(long, value_enum)]
    pub progress: Option<ProgressMode>,
}

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Save a validated PAT for the current host.
    Login(AuthLoginArgs),
    /// Report whether the current host has a saved PAT.
    Status,
    /// Delete the saved PAT for the current host.
    Logout,
}

#[derive(Debug, Args)]
pub struct AuthLoginArgs {
    /// Read the PAT from stdin instead of prompting interactively.
    #[arg(long, default_value_t = false)]
    pub token_stdin: bool,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print the per-user config path.
    Path,
    /// Print the effective config payload.
    Show,
}
