# Release Checklist

This checklist is for preparing and validating a crates.io release.

## Prepare

1. Confirm `Cargo.toml` has the intended version.
2. Confirm `CHANGELOG.md` includes a release entry.
3. Confirm `docs/releases/vX.Y.Z.md` is suitable to paste into a GitHub
   Release.
4. Confirm README installation snippets reference the intended crate version.
5. Confirm all public APIs added in the release are documented in README and
   rustdoc.

## Validate

Run the CI-equivalent checks:

```bash
RUSTFLAGS="-Dwarnings" cargo check --all-targets
cargo fmt --all -- --check
cargo test --all-targets
cargo doc --no-deps
```

Run the stronger local gate before publishing:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo package --allow-dirty
```

Inspect the generated package:

```bash
cargo package --list --allow-dirty
```

The package should include the source, tests, README, license, changelog, and
release docs. It should not include local build outputs, `.omx` state, or
target artifacts.

## Publish Dry Run

Run:

```bash
cargo publish --dry-run --allow-dirty
```

Use `--allow-dirty` only while validating an unreleased branch. For the actual
publish, use a clean tagged checkout and omit `--allow-dirty`.

## Publish

1. Ensure the release PR has merged.
2. Pull the latest `main`.
3. Verify the worktree is clean.
4. Run the validation commands again without `--allow-dirty` where possible.
5. Publish:

   ```bash
   cargo publish
   ```

6. Create and push the release tag:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

7. Create the GitHub Release using `docs/releases/vX.Y.Z.md`.
8. Confirm crates.io and docs.rs show the new version.
