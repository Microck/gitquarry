# Gitquarry Operator Playbook

## Contents

1. Command selector
2. Auth and host workflow
3. Search workflow
4. Inspect workflow
5. Output and scripting rules
6. Benchmark-backed discover heuristics
7. Common failure modes

## Command Selector

Use the narrowest command that matches the task:

| Task | Command | Why |
| --- | --- | --- |
| Save or inspect credentials | `gitquarry auth ...` | Auth is host-scoped and should be debugged separately from search behavior. |
| Find candidate repositories | `gitquarry search ...` | Search is the discovery entry point. |
| Inspect one known repository | `gitquarry inspect owner/repo` | Inspect is metadata-first and avoids search indirection. |
| Check config state | `gitquarry config path` or `gitquarry config show` | Config inspection is explicit and low risk. |

## Auth And Host Workflow

Start here when anything smells like an auth, host, or enterprise issue.

### Default auth path

```bash
gitquarry auth login
gitquarry auth status
```

Non-interactive:

```bash
printf '%s' "$GITHUB_TOKEN" | gitquarry auth login --token-stdin
```

Resolution order:

1. `GITQUARRY_TOKEN_<NORMALIZED_HOST>`
2. `GITQUARRY_TOKEN`
3. saved secure credential
4. explicit insecure-file fallback

### Host-scoped examples

GitHub.com:

```bash
export GITQUARRY_TOKEN_GITHUB_COM=ghp_example
gitquarry --host github.com search "rust cli"
```

GitHub Enterprise:

```bash
gitquarry --host https://ghe.example.com auth login
gitquarry --host https://ghe.example.com search "platform tooling" --org engineering
gitquarry --host https://ghe.example.com inspect engineering/internal-cli
```

Use `GITQUARRY_CONFIG_DIR` for isolated runs:

```bash
GITQUARRY_CONFIG_DIR="$(mktemp -d)" \
GITQUARRY_TOKEN="$GITHUB_TOKEN" \
gitquarry search "rust cli" --format compact --progress off
```

### Auth rules

- Treat credentials as host-scoped.
- Re-check the host before assuming the token is wrong.
- Do not rely on insecure fallback unless `GITQUARRY_ALLOW_INSECURE_STORAGE=1` is explicitly set.
- Use `auth status` before changing search flags when debugging access problems.

## Search Workflow

### 1. Start native

Use native search first unless the task explicitly requires enhanced behavior:

```bash
gitquarry search "<query>"
```

Add structured filters before escalating:

```bash
gitquarry search "<query>" --language rust --topic cli --sort stars
gitquarry search "<query>" --org vercel --min-stars 100
gitquarry search "<query>" --updated-within 30d
```

### 2. Use discover mode deliberately

Use discover mode only when the task needs:

- broader candidate collection
- ranking by `activity`, `quality`, or `blended`
- README-aware reranking
- explain output

Baseline discover form:

```bash
gitquarry search "<query>" --mode discover
```

Discover contract:

- non-native ranks require `--mode discover`
- `--mode discover` without `--rank` defaults to `blended`
- `--readme` is enrichment, not retrieval, and should stay explicit

### 3. Prefer structured flags over raw qualifiers

Good:

```bash
gitquarry search "vector database" --language rust --sort stars
```

Avoid:

```bash
gitquarry search "language:rust" --language go
```

If a raw qualifier and a structured flag overlap, gitquarry should fail clearly. Fix the command instead of guessing which side wins.

## Inspect Workflow

Use `inspect` when the repository is already known:

```bash
gitquarry inspect rust-lang/rust
gitquarry inspect rust-lang/rust --readme --format json
```

Default inspect output is metadata-first:

- full name and URL
- description
- stars and forks
- language, topics, license
- timestamps
- archived, template, and fork state
- open issues
- latest release and contributor count when available

Rules:

- repository input must be `owner/repo`
- use `--readme` only when README content is actually needed
- prefer `inspect` over `search` when the target repo is explicit

## Output And Scripting Rules

Use formats intentionally:

| Format | Use it for |
| --- | --- |
| `pretty` | direct terminal reading |
| `json` | structured automation and tooling |
| `compact` | minified machine pipelines or logs |
| `csv` | flat exports |

Script-safe examples:

```bash
gitquarry search "rust cli" --format json | jq '.items[].full_name'
gitquarry search "release automation" --mode discover --format compact --progress off | jq '.total_count'
gitquarry inspect rust-lang/rust --readme --format json | jq '.repository.latest_release.tag_name'
```

Rules:

- structured data should go to `stdout`
- progress and errors should stay on `stderr`
- prefer `--progress off` in CI and agent runs

## Benchmark-Backed Discover Heuristics

These are the decision-grade search presets from the live benchmark study. They are not the whole skill, but they are the best guidance when the task is specifically about choosing discover settings.

### Default recommendation

Use this when the operator wants a stronger-than-native result set without a long discussion:

```bash
gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain
```

Why:

- it was the best default non-native choice across the live study
- it preserved the native core much better than `query`
- it stayed in the practical latency band instead of jumping to deep-mode cost

### Latency ladder

| Path | Typical cost from study | What it buys |
| --- | --- | --- |
| Native | `~0.5s` to `~1.1s` | Pure GitHub baseline |
| Discover quick | `~16s` to `~18s` | Cheapest discover proof point |
| Discover balanced | `~27s` to `~30s` | Main tradeoff zone |
| Discover deep | `~53s` to `~60s` | High recall tax |
| README enrichment tax | `+3s` to `+5s` | Stronger evidence, not better top-10 overlap in this run |

### Intent-to-command mapping

| Intent | Command | Why |
| --- | --- | --- |
| Fastest baseline | `gitquarry search "<query>"` | Keeps native GitHub behavior and sub-second latency. |
| Safer advanced default | `gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain` | Best general upgrade from native without major drift. |
| Broader semantic exploration | `gitquarry search "<query>" --mode discover --depth balanced --rank query --explain` | Highest novelty in the balanced family. |
| Noisy infra-style query | `gitquarry search "<query>" --mode discover --depth balanced --rank blended --weight-query 0.5 --weight-activity 0.5 --weight-quality 2.0 --explain` | Best upgrade for the `api gateway` benchmark shape. |
| Fresh repositories | `gitquarry search "<query>" --mode discover --depth balanced --rank activity --updated-within 1y --explain` | Freshness is the intent, so churn is acceptable. |
| Language slice | `gitquarry search "<query>" --language Rust` | Cheap first pass. Add discover only if the slice still needs expansion. |
| README evidence pass | `gitquarry search "<query>" --mode discover --depth balanced --rank quality --readme --explain` | Use after a promising result set exists and stronger evidence is needed. |

### Query-family guidance

For `api gateway`-style noisy queries, prefer:

```bash
gitquarry search "<query>" --mode discover --depth balanced --rank blended --weight-query 0.5 --weight-activity 0.5 --weight-quality 2.0 --explain
```

For `terminal ui`-style cleaner queries, prefer:

```bash
gitquarry search "<query>" --mode discover --depth balanced --rank quality --explain
```

### Benchmark rules

- Do not present discover mode as near-native.
- Do not turn on `--readme` by default.
- Do not recommend balanced `query` as the safest advanced preset.
- Treat recency and language slices as different intents, not minor tweaks.

## Common Failure Modes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `E_AUTH_REQUIRED` | no effective token | run `auth login` or set the right env var |
| `E_AUTH_INVALID` | empty, malformed, or wrong-host token | verify token and host, then rerun login |
| `E_QUERY_REQUIRED` | native search without a query | add a query or use explicit discover mode with structured filters |
| `E_FLAG_REQUIRES_MODE` | discover-only flag used without `--mode discover` | add `--mode discover` |
| `E_FLAG_CONFLICT` | raw qualifier overlaps a structured flag, or repo shape is malformed | remove the conflict or fix `owner/repo` |
| `E_HOST_INVALID` | empty or malformed `--host` | pass a valid hostname or API root |

Use this troubleshooting sequence:

1. Verify host.
2. Verify auth state.
3. Verify command shape.
4. Verify flag compatibility.
5. Only then tune search behavior.
