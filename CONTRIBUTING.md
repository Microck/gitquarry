# Contributing

## Scope

`gitquarry` is intentionally narrow.

Contributions should stay focused on:

- the CLI surface
- repository and documentation quality
- verification and release tooling

Avoid broad speculative feature expansion that weakens the core contract around native search versus explicit discovery.

## Before You Change Behavior

Read these first:

- [Specification](docs/project/specification.mdx)
- [Architecture](docs/project/architecture.mdx)
- [README.md](README.md)

The main product rule is simple: plain search must stay close to native GitHub repository search. Enhanced behavior must remain explicit.

## Local Verification

Run the standard checks before opening a pull request:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Useful command-surface checks:

```bash
cargo run -- --help
cargo run -- search --help
cargo run -- auth login --help
```

## Documentation

If you change user-visible behavior:

- update [README.md](README.md)
- update the relevant page under [`docs/`](docs)
- keep the Mintlify docs aligned with the actual CLI help text

## Pull Requests

- Keep each pull request scoped to one goal
- Add or update tests when behavior changes
- Include verification notes in the pull request body
- Do not commit secrets, tokens, or local credential files

## Auth and Test Safety

Do not commit:

- `.env` files
- PATs
- local config with credentials
- captured tokens in test fixtures

Prefer deterministic local tests over live authenticated runs. If a change genuinely needs live verification, document the exact manual steps and expected result.

## Release Notes

For notable user-facing changes, add a short entry to [CHANGELOG.md](CHANGELOG.md).

If a change is behaviorally breaking, call that out explicitly.

## Code of Conduct

By participating in this project, you agree to follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
