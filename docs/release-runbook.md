# Release Runbook

Use this when cutting a new `gitquarry` release.

## Goal

Ship one version across GitHub release assets, downstream packaging repos, public documentation, and crates.io when registry credentials are configured.

## Preflight

1. Merge the approved work into `main`.
2. Make sure `main` is green before tagging.
3. Pick the release version `X.Y.Z`.
4. Confirm release automation credentials are present:
   - `CARGO_REGISTRY_TOKEN` GitHub Actions secret for crates.io publishing
   - `GITQUARRY_TOKEN` GitHub Actions secret for live smoke checks
5. Confirm `CHANGELOG.md` includes the user-facing notes for the release.
6. Be ready to update downstream packaging repos after the GitHub release completes:
   - `Microck/homebrew-gitquarry`
   - `Microck/scoop-gitquarry`

## Update release metadata

1. Bump the release version in:
   - `Cargo.toml`
   - `Cargo.lock`
2. Prefer moving the notes from `## Unreleased` into a release-specific `## [X.Y.Z]` section for versioned history.
3. If you intentionally leave the notes under `## Unreleased`, the workflow will use that section as a fallback release summary.
4. Check for hardcoded version references in docs or examples.
5. Commit the release metadata update on `main`.

## Local verification before tagging

Run the same checks the release pipeline depends on:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo package --locked
cargo publish --locked --dry-run
```

If any command fails, fix it before tagging.

## Publish the release

1. Push `main`.
2. Create and push the release tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

## What the tag triggers

`.github/workflows/release.yml` runs on `v*` tags and:

- verifies tag-version alignment
- runs format, clippy, tests, and package checks
- publishes the crate to crates.io when `CARGO_REGISTRY_TOKEN` is configured
- builds release artifacts for:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- creates a source tarball for packaging workflows
- generates `SHA256SUMS`
- creates the GitHub release and uploads the packaged artifacts

## Post-tag checks

Verify all public release surfaces after the workflows finish:

1. GitHub Release
   - `gh release view vX.Y.Z -R Microck/gitquarry`
   - confirm the release includes the per-target archives, the source tarball, and `SHA256SUMS`
2. Release workflow health
   - `gh run list --workflow Release -R Microck/gitquarry --limit 5`
3. Downstream packaging repos
   - update `Microck/homebrew-gitquarry/Formula/gitquarry.rb` to the new version and hashes
   - update `Microck/scoop-gitquarry/bucket/gitquarry.json` to the new version and Windows hash
   - push both repos and verify the install commands still work conceptually:

```bash
brew tap Microck/gitquarry
brew install gitquarry

scoop bucket add gitquarry https://github.com/Microck/scoop-gitquarry
scoop install gitquarry
```

4. Repo-native packaging files
   - update `packaging/homebrew/gitquarry.rb`
   - update `packaging/scoop/gitquarry.json`
   - update `packaging/aur/PKGBUILD` and `packaging/aur/.SRCINFO`
5. crates.io
   - `cargo search gitquarry --limit 1`
   - confirm the published version matches `X.Y.Z` when `CARGO_REGISTRY_TOKEN` was configured for the release
6. Docs
   - confirm Mintlify is still pointing at the expected branch and that install instructions still match the published release

## Release channel notes

### GitHub Releases

This is the canonical binary distribution channel for platform archives.

### Homebrew

This lives in `Microck/homebrew-gitquarry` and should always track the latest published GitHub release hashes for macOS and Linux.

### Scoop

This lives in `Microck/scoop-gitquarry` and should always track the latest published Windows release asset and hash.

### Repo-native packaging files

The main repo also carries release-aligned packaging files under `packaging/` for Homebrew, Scoop, and AUR workflows. They should be kept in sync with the public release even if an external registry has not been updated yet.

### crates.io

This is the Rust package distribution channel when `CARGO_REGISTRY_TOKEN` is configured. `cargo install gitquarry` depends on the crates.io version being in sync with the GitHub tag.

### Mintlify

Mintlify docs are source-controlled in this repo. There is no version-specific docs publish step in the workflow, so any docs changes should already be merged before the release tag is pushed.

## Recovery paths

### Tag exists but the workflow failed

1. Inspect the failed run:

```bash
gh run list --workflow Release -R Microck/gitquarry --limit 5
gh run view <run-id> -R Microck/gitquarry --log
```

2. Fix the underlying issue on `main`.
3. Decide whether to:
   - delete and recreate the tag before any public publish happened, or
   - cut a new patch version if the release already escaped externally

### GitHub release exists but crates.io publish failed

1. Verify `CARGO_REGISTRY_TOKEN` is configured.
2. Re-run the publish step manually from a trusted local environment:

```bash
cargo publish --locked
```

3. Confirm crates.io shows the new version before announcing the release.

### crates.io is published but GitHub assets are wrong

1. Rebuild the release artifacts from the matching tag.
2. Replace the GitHub release assets.
3. Do not republish the crate with the same version.

### GitHub release is good but downstream package repos drifted

1. Download `SHA256SUMS` from the GitHub release.
2. Update `Microck/homebrew-gitquarry` and `Microck/scoop-gitquarry` to the matching version and hashes.
3. Update the repo-native files under `packaging/`.
4. Verify docs still point at live channels only.

## Quick checks

- `gh release view vX.Y.Z -R Microck/gitquarry`
- `gh run list --workflow Release -R Microck/gitquarry --limit 5`
- `gh release download vX.Y.Z -R Microck/gitquarry -p SHA256SUMS`
- `cargo search gitquarry --limit 1`
