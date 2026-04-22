<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/gitquarry-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset=".github/assets/gitquarry-logo-light.svg">
    <img src=".github/assets/gitquarry-logo-light.svg" alt="gitquarry" width="720">
  </picture>
</p>


<p align="center">
  <a href="https://github.com/Microck/gitquarry/releases"><img src="https://img.shields.io/github/v/release/Microck/gitquarry?display_name=tag&style=flat-square&label=release&color=000000" alt="release badge"></a>
  <a href="https://github.com/Microck/gitquarry/releases"><img src="https://img.shields.io/github/downloads/Microck/gitquarry/total?style=flat-square&label=downloads&color=000000" alt="downloads badge"></a>
  <a href="https://github.com/Microck/gitquarry/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Microck/gitquarry/ci.yml?branch=main&style=flat-square&label=ci&color=000000" alt="ci badge"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-mit-000000?style=flat-square" alt="license badge"></a>
</p>

---

`gitquarry` is a terminal CLI for GitHub repository search that keeps native search behavior intact by default and only turns on broader discovery, reranking, and README-aware enrichment when you ask for them. it is built for people who want a practical command surface for ad hoc exploration, shell workflows, and structured output without hiding what the underlying GitHub search call is doing.

the main setup path is `gitquarry auth login`. on a real terminal it gives you a direct way to save a GitHub personal access token, validate it, and then use the same credential flow for normal search commands, repository inspection, and scripted JSON output. host-specific environment overrides and an insecure local fallback path also exist when you need them, but they stay explicit.

[documentation](./docs/index.mdx) | [releases](https://github.com/Microck/gitquarry/releases) | [github](https://github.com/Microck/gitquarry)

## why

if you already use GitHub repository search and want a CLI that stays honest about what is native versus what is enhanced, this gives you that path without silently turning every query into a ranking engine.

- native search stays native by default, including GitHub-style ordering and query semantics
- discover mode is explicit, so broader candidate collection and reranking only happen when you opt in
- README enrichment is off unless you request it
- output modes are practical for both humans and scripts: `pretty`, `json`, `compact`, and `csv`
- auth handling is host-aware and works with secure storage, env overrides, or an explicit insecure fallback
- failure modes are intended to be clear, narrow, and script-friendly

## install

the public install path right now is GitHub Releases:

- [releases page](https://github.com/Microck/gitquarry/releases)
- `gitquarry-linux.tar.gz`
- `gitquarry-macos.tar.gz`
- `gitquarry-windows.zip`

if you want to install from a local checkout instead:

```bash
cargo install --path .
```

if you just want to run it from source:

```bash
cargo run -- search "rust cli"
```

## auth

interactive login:

```bash
gitquarry auth login
```

stdin login:

```bash
printf '%s' "$GITHUB_TOKEN" | gitquarry auth login --token-stdin
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

status and logout:

```bash
gitquarry auth status
gitquarry auth logout
```

secure storage is the default path. if the local keyring is unavailable and you explicitly opt in with `GITQUARRY_ALLOW_INSECURE_STORAGE=1`, gitquarry can fall back to a local credential file with owner-only permissions on Unix-like systems.

## search

native search:

```bash
gitquarry search "rust cli"
gitquarry search "vector database" --sort stars
gitquarry search "vector database" --language rust --topic cli
```

explicit discovery and reranking:

```bash
gitquarry search "rust cli" \
  --mode discover \
  --rank blended \
  --readme \
  --explain

gitquarry search --mode discover --topic cli --updated-within 30d
gitquarry search "graphql client" --mode discover --rank activity
```

repository inspection:

```bash
gitquarry inspect rust-lang/rust --format json
gitquarry inspect owner/repo --readme --format csv
```

## output

both `search` and `inspect` support:

- `pretty`
- `json`
- `compact`
- `csv`

examples:

```bash
gitquarry search "rust cli" --format json
gitquarry search "release automation" --mode discover --format compact
gitquarry inspect rust-lang/rust --readme --format csv
```

## docs

the docs site source lives in [`docs/`](./docs) and is written for Mintlify.

good starting points:

- [`docs/index.mdx`](./docs/index.mdx)
- [`docs/guides/quickstart.mdx`](./docs/guides/quickstart.mdx)
- [`docs/guides/authentication.mdx`](./docs/guides/authentication.mdx)
- [`docs/commands/search.mdx`](./docs/commands/search.mdx)
- [`docs/reference/output-contract.mdx`](./docs/reference/output-contract.mdx)

local preview:

```bash
cd docs
mint dev
```

## development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## status

`gitquarry` is intentionally narrow right now:

- public repositories only
- PAT-only authentication
- REST-only GitHub integration
- GitHub.com first, custom hosts best effort

that constraint is part of the product design, not an accident.
