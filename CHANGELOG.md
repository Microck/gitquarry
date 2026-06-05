# Changelog

All notable user-facing changes to `gitquarry` should be documented in this file.

The format is intentionally simple and does not depend on a release tool.

## Unreleased

## [0.1.9]

- Add `gitquarry agent` and `gitquarry skills` so agents can load embedded, version-matched CLI usage guidance from the installed binary

## [0.1.8]

- Add `--format toon` for compact structured CLI output across search, inspect, tree, and code commands
- Document TOON in command docs, the output/scripting guide, output contracts, config reference, and project specification
- Keep CI from restoring cached Cargo binaries on macOS runners, avoiding stale `cargo` shims from restored caches

## [0.1.7]

- Add `gitquarry tree` for no-clone repository tree inspection with path glob, text, depth, ref, and structured output controls
- Add `gitquarry code` for no-clone repository code search with literal or regex matching, path filters, context lines, limits, and structured output
- Document tree and code commands for CLI, MCP, and agent workflows

## [0.1.6]

- Keep GitHub release publishing green when npm registry permissions reject wrapper publish
- Republish the source retrieval release with non-blocking npm publishing

## [0.1.5]

- Add `gitquarry source path` for explicit `opensrc`-backed source retrieval
- Document source retrieval behavior, output shape, and failure codes
- Fix release workflow Rust target installation to match the pinned toolchain

## [0.1.4]

- Attempted release superseded by `0.1.5` before public release artifacts were published

## [0.1.3]

- Move the Intel macOS release build onto `macos-latest` to avoid blocked `macos-13` runner capacity

## [0.1.2]

- Add package-manager-friendly release assets with explicit target triples and a source tarball
- Add repo-native Nix packaging and prepare the repository for broader distribution channels

## [0.1.1]

- Fix the tagged release workflow so release notes generation can read `CHANGELOG.md`
- Update GitHub Actions workflow dependencies to Node 24 compatible releases

## [0.1.0]

- Initial public-repo preparation
- Mintlify documentation scaffold
- CI, release, and live-smoke workflow setup
- Crates.io publish wiring, package metadata hardening, and public repo polish
- Auth and contributor-count edge case fixes from live verification
