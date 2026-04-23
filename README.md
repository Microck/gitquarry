<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/gitquarry-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset=".github/assets/gitquarry-logo-light.svg">
    <img src=".github/assets/gitquarry-logo-light.svg" alt="gitquarry" width="720">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/Microck/gitquarry/releases"><img src="https://img.shields.io/github/v/release/Microck/gitquarry?display_name=tag&style=flat-square&label=release&color=000000" alt="release badge"></a>
  <a href="https://www.npmjs.com/package/gitquarry"><img src="https://img.shields.io/npm/dt/gitquarry?style=flat-square&label=downloads&color=000000" alt="npm downloads"></a>
  <a href="https://github.com/Microck/gitquarry/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Microck/gitquarry/ci.yml?branch=main&style=flat-square&label=ci&color=000000" alt="ci badge"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-mit-000000?style=flat-square" alt="license badge"></a>
</p>

---

`gitquarry` is a terminal CLI for GitHub repository search that keeps native search behavior intact by default and only turns on broader discovery, reranking, and README-aware enrichment when you ask for them. it is built for people who want one command surface for interactive use, shell workflows, and structured output without hiding what the underlying GitHub search call is doing.

the main setup path is `gitquarry auth login`. on a real terminal it prompts for a GitHub personal access token, validates it immediately, saves it for the current host, and then uses that same host-scoped credential flow for normal search commands, repository inspection, and scripted JSON output. if you prefer environment-based auth, host-specific overrides and an explicit insecure fallback path also exist, but they stay explicit.

[documentation](./docs/index.mdx) | [npm](https://www.npmjs.com/package/gitquarry) | [github](https://github.com/Microck/gitquarry)

## why

if you already use GitHub repository search and want a CLI that stays honest about what is native versus what is enhanced, this gives you that path without silently turning every query into a ranking engine.

- native search stays native by default, including GitHub-style ordering and query semantics
- discover mode is explicit, so broader candidate collection and reranking only happen when you opt in
- README enrichment is off unless you request it
- output modes are practical for both humans and scripts: `pretty`, `json`, `compact`, and `csv`
- auth handling is host-aware and works with secure storage, env overrides, or an explicit insecure fallback
- failure modes are intended to be clear, narrow, and script-friendly

## quickstart

### Linux or macOS

```bash
npm install -g gitquarry
gitquarry auth login
gitquarry search "rust cli"
```

if you prefer native binaries over the npm wrapper, use the [GitHub Releases page](https://github.com/Microck/gitquarry/releases) or the Homebrew tap shown below.

### Windows

```powershell
scoop bucket add gitquarry https://github.com/Microck/scoop-gitquarry
scoop install gitquarry
gitquarry auth login
gitquarry search "rust cli"
```

the npm wrapper also works on Windows:

```powershell
npm install -g gitquarry
```

### using a package manager

```bash
npm install -g gitquarry
pnpm add -g gitquarry
bun add -g gitquarry

# homebrew
brew tap Microck/gitquarry
brew install gitquarry

# scoop
scoop bucket add gitquarry https://github.com/Microck/scoop-gitquarry
scoop install gitquarry

# AUR (arch linux)
yay -S gitquarry

# nix
nix run github:Microck/gitquarry
```

direct platform archives and checksums are also published on the [releases page](https://github.com/Microck/gitquarry/releases).

### auth

run the interactive login:

```bash
gitquarry auth login
```

that path:

- reads a PAT from your terminal
- validates it immediately against the current host
- saves it into secure storage when available
- uses host-scoped credential resolution rather than one global token bucket

non-interactive alternative:

```bash
printf '%s' "$GITHUB_TOKEN" | gitquarry auth login --token-stdin
gitquarry auth status
```

environment override:

```bash
export GITQUARRY_TOKEN=ghp_your_token_here
gitquarry search "rust cli"
```

host-scoped environment override:

```bash
export GITQUARRY_TOKEN_GITHUB_COM=ghp_your_token_here
gitquarry --host github.com search "rust cli"
```

if secure storage is unavailable and you explicitly want the fallback path, opt in first:

```bash
export GITQUARRY_ALLOW_INSECURE_STORAGE=1
gitquarry auth login
```

## auth model

| credential path | what it unlocks |
| --- | --- |
| `GITQUARRY_TOKEN_<NORMALIZED_HOST>` | highest-precedence credential for one host, used by `search` and `inspect` for that host |
| `GITQUARRY_TOKEN` | global env fallback when no host-specific env var is present |
| saved secure credential | normal `auth login` path for `search`, `inspect`, and host-aware auth commands |
| explicit insecure fallback file | only used when secure storage is unavailable and `GITQUARRY_ALLOW_INSECURE_STORAGE=1` is set |
| none | `--help`, `version`, `config path`, `config show`, and auth-management commands still work |

notes:

- credentials are host-scoped
- environment variables override saved credentials
- the token is validated before gitquarry claims login succeeded
- secure OS storage is the default path
- insecure fallback is opt-in only
- `auth logout` removes saved credentials for the current host, including the insecure-file fallback when present

for GitHub.com, the host-specific env var is:

```bash
export GITQUARRY_TOKEN_GITHUB_COM=ghp_your_token_here
```

for a GitHub Enterprise host, gitquarry derives the env var name from the normalized host and keeps the credential resolution local to that host.

## command surface

| command | purpose |
| --- | --- |
| `gitquarry search` | search public repositories with native mode by default and explicit discover mode when requested |
| `gitquarry inspect` | inspect one explicit `owner/repo` with optional README inclusion |
| `gitquarry auth` | save, inspect, and remove host-scoped personal access tokens |
| `gitquarry config` | print the effective config path or effective config payload |
| `gitquarry version` | print the current gitquarry version |

for automation, stdout stays in one of the supported formats and progress stays on `stderr`.

supported formats:

- `pretty`
- `json`
- `compact`
- `csv`

root commands:

```bash
gitquarry search [OPTIONS] [QUERY]
gitquarry inspect [OPTIONS] <OWNER/REPO>
gitquarry auth login|status|logout
gitquarry config path|show
gitquarry version
gitquarry --generate-completion <shell>
```

search-specific controls include:

- `--mode native|discover`
- `--rank native|query|activity|quality|blended`
- `--sort best-match|stars|updated`
- structured filters like `--language`, `--topic`, `--license`, `--user`, `--org`
- range filters like `--min-stars`, `--max-stars`, `--min-forks`, `--max-forks`
- date filters like `--created-after`, `--updated-before`, `--pushed-within`
- enhancement flags like `--readme`, `--explain`, and blended ranking weights

inspect-specific controls stay intentionally narrow:

- `--readme`
- `--format pretty|json|compact|csv`
- `--progress auto|on|off`

config is intentionally conservative. saved defaults can cover things like `host`, `format`, `limit`, `progress`, and `color`, but not flags that would silently enable enhanced search behavior.

## shell completion

generate a completion script and install it with your shell of choice:

```bash
# bash
gitquarry --generate-completion bash > ~/.local/share/bash-completion/completions/gitquarry

# zsh
gitquarry --generate-completion zsh > ~/.zsh/completion/_gitquarry

# fish
gitquarry --generate-completion fish > ~/.config/fish/completions/gitquarry.fish

# powershell
gitquarry --generate-completion powershell >> $PROFILE
```

see the [installation guide](./docs/guides/installation.mdx) for the fuller install matrix.

## examples

use search as part of a shell workflow:

```bash
gitquarry search "rust cli"
```

switch the same command to terminal-readable output:

```bash
gitquarry search --format pretty "rust cli"
```

keep native GitHub-like ordering but add structured filters:

```bash
gitquarry search "vector database" --language rust --topic cli --sort stars
```

run discover mode explicitly:

```bash
gitquarry search "release automation" --mode discover
```

turn on blended reranking with README-aware scoring:

```bash
gitquarry search "release automation" \
  --mode discover \
  --rank blended \
  --readme \
  --explain
```

run discover mode without a free-text query and only structured filters:

```bash
gitquarry search --mode discover --topic cli --updated-within 30d --limit 5
```

use explicit ranking weights:

```bash
gitquarry search "graphql client" \
  --mode discover \
  --rank blended \
  --weight-query 1.5 \
  --weight-activity 0.8 \
  --weight-quality 1.2
```

inspect one repository in human-readable form:

```bash
gitquarry inspect rust-lang/rust
```

inspect the same repository with README content in JSON:

```bash
gitquarry inspect rust-lang/rust --readme --format json
```

pipe structured output without polluting stdout with progress:

```bash
gitquarry search "rust cli" --format json | jq '.items[0].full_name'
```

inspect the effective config and current config path:

```bash
gitquarry config show
gitquarry config path
```

switch hosts explicitly for GitHub Enterprise:

```bash
gitquarry --host https://ghe.example.com auth login
gitquarry --host https://ghe.example.com search "internal platform"
```

## benchmark snapshot

the repository includes a full benchmark study for `gitquarry search` under [docs/project/benchmark-study.mdx](./docs/project/benchmark-study.mdx). it compares native search, discover depth, rank modes, README enrichment, weighted blends, and recency/language slices on live GitHub data.

high-level findings from the current study:

- native is still the only sub-second path
- quick discover adds roughly `~16s` to `~18s`
- balanced discover adds roughly `~27s` to `~30s`
- deep discover adds roughly `~53s` to `~60s`
- README enrichment added another `~3s` to `~5s` on top of balanced discover in the benchmark queries
- `--mode discover --depth balanced --rank quality --explain` is the best default upgrade when you want smarter curation without throwing away the native core
- `--mode discover --depth balanced --rank query --explain` is the better fit when you explicitly want more novel repositories

practical operator rule:

```bash
# safest advanced default
gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain

# broader semantic expansion
gitquarry search "<query>" --mode discover --depth balanced --rank query --explain
```

the benchmark docs include the full recommendation matrix, query-specific findings, visual study assets, and the reproducible benchmark/showcase pipeline.

## what it looks like

if you want a quick feel for the CLI before installing it, this is the kind of command surface it exposes:

```text
Usage: gitquarry [OPTIONS] [COMMAND]

Commands:
  search   Search public repositories
  inspect  Inspect one explicit owner/repo
  auth     Manage host-scoped personal access tokens
  config   Show config path or the effective config payload
  version  Print the current gitquarry version
```

for day-to-day use, the main output shapes are:

- `pretty` for terminal scanning
- `json` for stable structured output
- `compact` for minified pipeline-oriented JSON
- `csv` for flat export workflows

that means the same command can move from terminal use to scripts without changing the command family:

```bash
gitquarry search "rust cli" --format pretty
gitquarry search "rust cli" --format json
gitquarry search "rust cli" --format compact
gitquarry inspect rust-lang/rust --format csv
```

the important operational rule is that progress output stays on `stderr`, so structured stdout remains safe in pipelines.

## building from source

if you are working on the CLI itself, build from a local checkout:

```bash
git clone https://github.com/Microck/gitquarry.git
cd gitquarry
cargo build --release
./target/release/gitquarry --help
```

common local checks:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

if you want to run it from source without installing it globally:

```bash
cargo run -- search "rust cli"
```

for the fuller install matrix and release operations, use the [installation guide](./docs/guides/installation.mdx) and the [release runbook](./docs/release-runbook.md).

## documentation

- [introduction](./docs/index.mdx)
- [installation guide](./docs/guides/installation.mdx)
- [quickstart guide](./docs/guides/quickstart.mdx)
- [authentication guide](./docs/guides/authentication.mdx)
- [discovery mode guide](./docs/guides/discovery-mode.mdx)
- [search command reference](./docs/commands/search.mdx)
- [inspect command reference](./docs/commands/inspect.mdx)
- [auth command reference](./docs/commands/auth.mdx)
- [config command reference](./docs/commands/config.mdx)
- [output contract](./docs/reference/output-contract.mdx)
- [error reference](./docs/reference/error-reference.mdx)
- [search behavior reference](./docs/reference/search-behavior.mdx)

## contributing

contributions are welcome. please open an issue or pull request on [github](https://github.com/Microck/gitquarry). for behavior changes, start by reading:

- [specification](./docs/project/specification.mdx)
- [architecture](./docs/project/architecture.mdx)

if you change the command surface or user-visible behavior:

- update the README
- update the relevant Mintlify page under `docs/`
- keep the docs aligned with actual CLI help output

the repository also includes issue templates for bug reports and feature requests under [`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE).

## disclaimer

this project is unofficial and not affiliated with, endorsed by, or connected to GitHub, Inc. it is an independent, community-built tool.

## license

[mit license](LICENSE)
