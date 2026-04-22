# gitquarry v1 plan

## summary

`gitquarry` is a PAT-authenticated Rust CLI for searching public GitHub repositories.
Its default behavior is intentionally native and boring: plain `search "query"` should feel as close to GitHub repository search as possible.
All enhanced discovery behavior is explicit and flag-driven.

## delivery phases

### phase 1 - foundation

- Build the root CLI surface with:
  - `search`
  - `inspect`
  - `auth login|status|logout`
  - `config path|show`
  - `version` and root `--version`
  - root `--generate-completion <shell>`
- Implement host normalization and host-scoped auth resolution.
- Implement config loading plus secure credential storage.
- Add symbolic error codes and fail-fast validation.

### phase 2 - native search

- Implement REST-only repository search against GitHub.com by default, with best-effort custom host support via `--host`.
- Support the structured native filter flag set and compile it into GitHub-compatible search queries where possible.
- Keep default output `pretty`, plus `json|compact|csv`.
- Enforce conflict rules between raw query qualifiers and overlapping structured flags.

### phase 3 - enhanced discovery

- Add explicit `--mode discover`.
- Add explicit rank modes: `native|query|activity|quality|blended`.
- Add bounded discovery depth, candidate-pool collection, reranking, and README enrichment behind explicit flags only.
- Add `--explain` and weighted blended ranking.

### phase 4 - inspect, polish, docs, tests

- Implement `inspect owner/repo` with metadata-first output and optional `--readme`.
- Add progress reporting to `stderr` for long-running enhanced commands.
- Add shell completion generation and command help polish.
- Add documentation and verification coverage for auth, config, output contracts, and error codes.

## acceptance criteria

- No-flags `search` performs one native GitHub repository search and preserves native ordering.
- Enhanced behavior never happens silently.
- PAT auth is always required and is stored securely by default.
- Structured outputs are stable enough for agents and scripts.
- Invalid flag combinations fail with one clear stderr message and a symbolic error code.
- Discovery mode remains bounded, explicit, and per-command only.

## assumptions

- Working product name is `gitquarry`.
- v1 is public-repos-only.
- v1 is REST-only.
- v1 supports GitHub.com first and custom hosts on a best-effort basis.
