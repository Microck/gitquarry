<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/gitquarry-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset=".github/assets/gitquarry-logo-light.svg">
    <img src=".github/assets/gitquarry-logo-light.svg" alt="gitquarry" width="760">
  </picture>
</p>

<p align="center">
  Search GitHub repositories from the terminal without hiding the native model. <br>
  Use plain native search by default. Turn on discovery and reranking only when you mean to.
</p>

<p align="center">
  <a href="https://github.com/Microck/gitquarry/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/Microck/gitquarry/ci.yml?branch=main&label=ci"></a>
  <a href="https://crates.io/crates/gitquarry"><img alt="crates.io" src="https://img.shields.io/crates/v/gitquarry"></a>
  <a href="https://docs.rs/gitquarry"><img alt="docs.rs" src="https://img.shields.io/docsrs/gitquarry"></a>
  <a href="https://github.com/Microck/gitquarry/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/gitquarry"></a>
</p>

<p align="center">
  <a href="./docs/index.mdx">Documentation</a> |
  <a href="./SPEC.md">Spec</a> |
  <a href="./ARCHITECTURE.md">Architecture</a>
</p>

---

`gitquarry` is a Rust CLI for public GitHub repository search.

The core contract is simple:

- `gitquarry search "query"` stays close to native GitHub repository search
- enhanced retrieval is explicit through `--mode discover`
- enrichment like README fetches and reranking never happens silently

## Why Gitquarry

Most GitHub search tools either stay too thin to be useful or become opinionated ranking engines the moment you install them. `gitquarry` takes a harder line:

- Native path first: plain search does one GitHub repository search and preserves native ordering
- Discovery stays explicit: broader candidate collection, reranking, and README-aware scoring only activate when you ask for them
- Scriptable output: `pretty`, `json`, `compact`, and `csv` are built in
- Host-scoped auth: PATs resolve per host, with env overrides and secure storage by default
- Fail-fast behavior: invalid flag combinations produce one clear error code and stop

## Quick Look

```bash
# Native GitHub-style repository search
gitquarry search "rust cli"

# Explicit discovery and reranking
gitquarry search "rust cli" \
  --mode discover \
  --rank blended \
  --readme \
  --explain

# One explicit repository
gitquarry inspect rust-lang/rust --format json
```

## Installation

Once the crate is published, install it directly from crates.io:

```bash
cargo install gitquarry
```

Release binaries are also produced for tagged releases on GitHub:

- Linux: `gitquarry-linux.tar.gz`
- macOS: `gitquarry-macos.tar.gz`
- Windows: `gitquarry-windows.zip`

Download the matching archive from the GitHub Releases page for your platform.

If you are working from a local checkout, source install remains available:

```bash
# from a local clone of this repository
cargo install --path .
```

You can also run it directly from a checkout:

```bash
cargo run -- search "rust cli"
```

The repository includes CI, cross-platform release packaging, and crates.io publish wiring through the tag workflow.

## Authentication

GitHub authentication is always required.

Interactive login:

```bash
gitquarry auth login
```

Non-interactive login:

```bash
printf '%s' "$GITHUB_TOKEN" | gitquarry auth login --token-stdin
```

Environment override:

```bash
export GITQUARRY_TOKEN=ghp_your_token_here
gitquarry search "rust cli"
```

Host-scoped environment override:

```bash
export GITQUARRY_TOKEN_GITHUB_COM=ghp_your_token_here
gitquarry --host github.com search "rust cli"
```

Status and logout:

```bash
gitquarry auth status
gitquarry auth logout
```

Secure storage is the default path. If the local keyring is unavailable and you explicitly opt in with `GITQUARRY_ALLOW_INSECURE_STORAGE=1`, gitquarry can fall back to a local credential file with restricted owner-only permissions on Unix-like systems.

## Search Modes

### Native

Use native mode when you want GitHub semantics and GitHub ordering:

```bash
gitquarry search "vector database"
gitquarry search "vector database" --sort stars
gitquarry search "vector database" --language rust --topic cli
```

### Discover

Use discover mode when you want candidate pooling, enrichment, and reranking:

```bash
gitquarry search --mode discover --topic cli --updated-within 30d
gitquarry search "graphql client" --mode discover --rank activity
gitquarry search "release automation" --mode discover --rank blended --readme
```

Discovery is still bounded. It does not introduce a persistent index or background crawler.

## Output Formats

Both `search` and `inspect` support:

- `pretty`
- `json`
- `compact`
- `csv`

Examples:

```bash
gitquarry search "rust cli" --format json
gitquarry inspect owner/repo --readme --format csv
```

## Documentation

The full documentation site source lives in [`docs/`](./docs) and is written for Mintlify.

Suggested reading order:

- [`docs/index.mdx`](./docs/index.mdx) - product overview
- [`docs/guides/quickstart.mdx`](./docs/guides/quickstart.mdx) - first successful commands
- [`docs/guides/authentication.mdx`](./docs/guides/authentication.mdx) - PAT and storage behavior
- [`docs/commands/search.mdx`](./docs/commands/search.mdx) - native vs discover details
- [`docs/reference/output-contract.mdx`](./docs/reference/output-contract.mdx) - scripting contract

## Local Docs Preview

Mintlify now uses `docs.json` as the site config. This repo keeps it under [`docs/docs.json`](./docs/docs.json).

From the docs directory, preview the site with the current Mintlify CLI:

```bash
cd docs
mint dev
```

If you prefer not to install the CLI globally, follow the current Mintlify CLI install instructions and run the local dev command from the `docs/` directory where `docs.json` lives.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Status

`gitquarry` is intentionally narrow right now:

- public repositories only
- PAT-only authentication
- REST-only GitHub integration
- GitHub.com first, custom hosts best effort

That constraint is part of the product design, not an accident.
