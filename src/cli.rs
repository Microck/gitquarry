use crate::model::{
    BoolFlag, DiscoveryDepth, ForkMode, OutputFormat, ProgressMode, RankMode, RetrievalMode,
    SearchPatternMode, SearchSort,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

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
    about = "Search public GitHub repositories with explicit discovery controls.",
    long_about = "Search public GitHub repositories with explicit discovery controls.

Agent usage:
    gitquarry skills get gitquarry

    Skills ship with the CLI and are always version-matched. They include
    workflow patterns, search mode guidance, auth rules, and copy-paste
    examples. Prefer this over guessing commands from flag docs alone.

    skills [list]              List available skills
    skills get gitquarry       Core CLI usage guide
    skills get <name>          Load a specialized skill
    skills path [name]         Print the embedded skill locator"
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
    /// Print the embedded agent skill guide for using gitquarry.
    Agent,
    /// List and load embedded agent skills.
    Skills(SkillsArgs),
    /// Inspect one explicit owner/repo.
    Inspect(InspectArgs),
    /// Print a repository file tree without cloning it.
    Tree(TreeArgs),
    /// Search repository code without cloning it.
    Code(CodeArgs),
    /// Compare explicit repositories side by side.
    Compare(CompareArgs),
    /// Fetch or locate source code through opensrc.
    Source(SourceArgs),
    /// Run checked-in search recipes.
    Recipe(RecipeArgs),
    /// Run a stdio Model Context Protocol server exposing gitquarry tools.
    Mcp(McpArgs),
    /// Manage host-scoped personal access tokens.
    Auth(AuthArgs),
    /// Show config path or the effective config payload.
    Config(ConfigArgs),
    /// Print the current gitquarry version.
    Version,
}

#[derive(Debug, Args)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: Option<SkillsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum SkillsCommand {
    /// List available embedded skills.
    List,
    /// Print an embedded skill guide.
    Get(SkillGetArgs),
    /// Print the embedded skill locator.
    Path(SkillPathArgs),
}

#[derive(Debug, Args)]
pub struct SkillGetArgs {
    /// Skill name.
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Include all embedded reference material for this skill.
    #[arg(long)]
    pub full: bool,
}

#[derive(Debug, Args)]
pub struct SkillPathArgs {
    /// Skill name; defaults to gitquarry.
    #[arg(value_name = "NAME")]
    pub name: Option<String>,
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

    /// Print the compiled search plan without calling GitHub.
    #[arg(long, default_value_t = false)]
    pub plan: bool,

    /// Probe result repositories for paths matching this glob. Repeat for OR semantics.
    #[arg(long = "probe-path")]
    pub probe_paths: Vec<String>,

    /// Probe result repositories for this literal or regex code pattern. Repeat for OR semantics.
    #[arg(long = "probe-code")]
    pub probe_code: Vec<String>,

    /// Treat probe code patterns as literal text or Rust regex.
    #[arg(long, value_enum, default_value_t = SearchPatternMode::Literal)]
    pub probe_mode: SearchPatternMode,

    /// Lines of context around probe code matches.
    #[arg(long, default_value_t = 0)]
    pub probe_context: usize,

    /// Maximum files to fetch per repository while probing code.
    #[arg(long, default_value_t = 20)]
    pub probe_limit: usize,

    /// Maximum code matches to record per repository while probing.
    #[arg(long, default_value_t = 100)]
    pub probe_match_limit: usize,

    /// Maximum file size to fetch during code probing.
    #[arg(long, default_value_t = 1_000_000)]
    pub probe_max_file_bytes: u64,
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

#[derive(Debug, Args, Clone)]
pub struct TreeArgs {
    /// Explicit repository identifier in owner/repo form.
    pub repository: String,

    /// Git ref to inspect. Defaults to the repository default branch.
    #[arg(long)]
    pub reference: Option<String>,

    /// Only show paths matching this glob pattern. Repeat for OR semantics.
    #[arg(long = "path")]
    pub paths: Vec<String>,

    /// Only show paths containing this text.
    #[arg(long)]
    pub contains: Option<String>,

    /// Maximum path depth to print, where root entries are depth 1.
    #[arg(long)]
    pub depth: Option<usize>,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Progress output mode for stderr.
    #[arg(long, value_enum)]
    pub progress: Option<ProgressMode>,
}

#[derive(Debug, Args, Clone)]
pub struct CodeArgs {
    /// Explicit repository identifier in owner/repo form.
    pub repository: String,

    /// Text or regex pattern to search for.
    pub pattern: String,

    /// Git ref to inspect. Defaults to the repository default branch.
    #[arg(long)]
    pub reference: Option<String>,

    /// Restrict searched files to this glob pattern. Repeat for OR semantics.
    #[arg(long = "path")]
    pub paths: Vec<String>,

    /// Treat the pattern as literal text or a Rust regex.
    #[arg(long, value_enum, default_value_t = SearchPatternMode::Literal)]
    pub mode: SearchPatternMode,

    /// Lines of context to include before and after each match.
    #[arg(long, default_value_t = 0)]
    pub context: usize,

    /// Maximum number of matches to print.
    #[arg(long, default_value_t = 100)]
    pub limit: usize,

    /// Maximum file size to fetch in bytes.
    #[arg(long, default_value_t = 1_000_000)]
    pub max_file_bytes: u64,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Progress output mode for stderr.
    #[arg(long, value_enum)]
    pub progress: Option<ProgressMode>,
}

#[derive(Debug, Args, Clone)]
pub struct CompareArgs {
    /// Explicit repository identifiers in owner/repo form.
    #[arg(required = true)]
    pub repositories: Vec<String>,

    /// Include each repository README in the output.
    #[arg(long, default_value_t = false)]
    pub readme: bool,

    /// Include a lightweight default-branch tree summary.
    #[arg(long, default_value_t = false)]
    pub tree_summary: bool,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Progress output mode for stderr.
    #[arg(long, value_enum)]
    pub progress: Option<ProgressMode>,
}

#[derive(Debug, Args)]
pub struct SourceArgs {
    #[command(subcommand)]
    pub command: SourceCommand,
}

#[derive(Debug, Subcommand)]
pub enum SourceCommand {
    /// Print the cached source path, fetching on cache miss.
    Path(SourcePathArgs),
}

#[derive(Debug, Args)]
pub struct RecipeArgs {
    #[command(subcommand)]
    pub command: RecipeCommand,
}

#[derive(Debug, Subcommand)]
pub enum RecipeCommand {
    /// Run a TOML search recipe.
    Run(RecipeRunArgs),
}

#[derive(Debug, Args, Clone)]
pub struct RecipeRunArgs {
    /// TOML recipe file to run.
    pub file: PathBuf,
}

#[derive(Debug, Args, Clone)]
pub struct McpArgs {}

#[derive(Debug, Args, Clone)]
pub struct SourcePathArgs {
    /// Package or repository spec accepted by opensrc, such as owner/repo or crates:serde.
    pub spec: String,

    /// Working directory for opensrc lockfile version resolution.
    #[arg(long)]
    pub cwd: Option<PathBuf>,

    /// Show opensrc fetch progress on stderr.
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
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
