# Changelog

All notable user-facing changes to `gitquarry` should be documented in this file.

The format is intentionally simple and does not depend on a release tool.

## Unreleased

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
