---
name: gitquarry-repository
description: Work effectively in the gitquarry repository. Use when changing gitquarry CLI behavior, search or inspect semantics, auth and host handling, output contracts, release automation, packaging, or project docs in this repo.
---

# Gitquarry Repository

## Read First

- Read `SPEC.md` before changing observable behavior.
- Read `ARCHITECTURE.md` before changing request flow, host handling, retries, or ranking internals.
- Read the relevant page under `docs/` before changing the command surface or output shape.
- Read `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `.github/workflows/live-smoke.yml` before changing release or verification logic.

## Preserve The Product Contract

- Keep plain `gitquarry search "query"` close to native GitHub repository search.
- Keep discover behavior explicit. Do not silently enable broader retrieval, reranking, or README enrichment.
- Keep validation ahead of auth resolution and network work when possible.
- Keep progress on `stderr` and structured output on `stdout`.
- Keep auth PAT-only and host-scoped in v1.
- Keep config conservative. Do not persist defaults that would silently enable enhanced behavior.

## Know The Important Source Files

- `src/app.rs` - top-level orchestration, command flow, README window handling, and render calls
- `src/query.rs` - flag validation, compiled query logic, conflict rules, empty-query behavior, and discovery planning
- `src/github.rs` - GitHub HTTP client, retries, API headers, README and release fetches, and contributor-count tolerance
- `src/credential.rs` - env resolution, secure storage, insecure fallback, and logout behavior
- `src/host.rs` - host normalization and host-specific token env-var naming
- `src/output.rs` - pretty, JSON, compact, and CSV rendering plus stdout/stderr guarantees
- `tests/cli-smoke.rs` - fixture-backed contract coverage for the public CLI
- `docs/` - Mintlify docs and wiki pages
- `npm/` - npm wrapper that downloads raw release binaries
- `packaging/` - repo-local Homebrew, Scoop, and AUR metadata mirrors

## Respect The Non-Obvious Rules

- Treat `updated-*` filters as post-filters. `created-*` and `pushed-*` compile into the GitHub query.
- Allow empty query only in discover-oriented usage. Native search without a query must fail.
- Default discover rank to `blended` and default discover depth to `balanced`.
- Keep discovery bounded. Do not introduce unbounded fan-out or persistent indexing without changing the contract docs.
- Treat `--concurrency` as advanced and conservative. Default is `1`.
- Treat README enrichment as enrichment-only. It must not change retrieval mode by itself.
- Tolerate contributor-count failures for very large repositories instead of failing the command.
- Use `GITQUARRY_CONFIG_DIR` for isolated local verification instead of touching a real user config directory.

## Verify Changes Properly

Run the standard checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo package --locked --allow-dirty
```

When release or npm-wrapper behavior changes, also run:

```bash
npm pack --dry-run ./npm
```

When the command surface changes, also run:

```bash
cargo run -- --help
cargo run -- search --help
cargo run -- inspect --help
cargo run -- auth login --help
```

## Prefer The Existing Test Strategy

- Prefer extending `tests/cli-smoke.rs` for end-to-end behavior.
- Use the in-process fixture HTTP server rather than mocks.
- Add the intended success path, the important failure path, and the motivating edge case.

## Keep Docs And Releases In Sync

- Update `README.md` when install, command, or product-positioning text changes.
- Update the relevant Mintlify page under `docs/` when behavior or outputs change.
- Update `docs/release-runbook.md` when release surfaces or sequencing change.
- Keep `Cargo.toml`, `Cargo.lock`, `npm/package.json`, and release tags aligned.
- Remember that release work may require syncing npm, Homebrew, Scoop, AUR, and repo-local packaging files.

## Avoid The Common Pitfalls

- Do not let discover behavior creep into the native path.
- Do not pollute `stdout` with progress or error chatter.
- Do not move validation after auth or network calls without a strong reason.
- Do not break host-scoped auth or config semantics.
- Do not let docs drift away from actual CLI help or release behavior.
