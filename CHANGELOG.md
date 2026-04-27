# Changelog

## 0.2.0 - 2026-04-27

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.0.md`](docs/releases/v0.2.0.md).

### Added

- Added the `jsonrepair` command-line binary for repairing stdin or files.
- Added `jsonrepair_to_writer` for writing repaired JSON to any
  `std::io::Write` destination.
- Added `jsonrepair_reader_to_writer` as the first streaming-oriented
  `Read -> Write` API.
- Added optional `serde` feature helpers:
  - `jsonrepair_value`
  - `jsonrepair_parse`
  - `JsonRepairParseError`
- Added `JsonRepairWriteError` and `JsonRepairStreamError` for distinguishing
  repair failures from IO failures.
- Added an upstream-style parity fixture corpus under `tests/fixtures/`.

### Changed

- Bumped the crate version to `0.2.0`.
- The CLI now uses the reader-to-writer repair API internally.

### Compatibility

- This release is intended to be source-compatible with `0.1.x` for existing
  `jsonrepair` callers.
- New public error enums are marked `#[non_exhaustive]`; downstream code should
  include fallback match arms.
- `jsonrepair_reader_to_writer` is an IO convenience MVP, not a true
  constant-memory streaming parser. It currently buffers complete input and
  repaired output inside the crate before writing.

### Validation

Release validation should run:

```bash
RUSTFLAGS="-Dwarnings" cargo check --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
cargo package --allow-dirty
cargo publish --dry-run --allow-dirty
```
