# Changelog

## 0.2.3 - 2026-09-25

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.3.md`](docs/releases/v0.2.3.md).

### Fixed

- Return an error for unsupported whitespace-only array values instead of
  looping without advancing the parser and exhausting memory. The case was
  found by the first manual run of the new fuzz workflow after 0.2.2 was published.
- Keep earlier root-list separators when removing the last pending separator,
  so successful repairs remain valid JSON.
- Escape control characters following an invalid backslash escape instead of
  returning a string containing invalid JSON control bytes.
- Avoid recursive string-repair retries on escaped commas followed by an
  ambiguous quote.
- Remove the pending root separator when a following fenced value is empty.

### Compatibility

- This is a source-compatible patch release. Public APIs and error types are
  unchanged.

## 0.2.2 - 2026-09-25

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.2.md`](docs/releases/v0.2.2.md).

### Fixed

- Reject plus-prefixed exponent-only tokens that cannot form valid JSON
  numbers (#65).
- Repair truncated LLM JSON string snapshots that end after the first slash of
  a URL-like `https:/` value.

### Added

- Add fuzz coverage for chunked reader-to-writer repair and scheduled fuzz runs.
- Add a reproducible CLI-level timing comparison with Python `json-repair`.

### Compatibility

- This is a source-compatible patch release. The reader-to-writer API still
  buffers internally; it does not perform constant-memory streaming.

## 0.2.1 - 2026-06-01

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.1.md`](docs/releases/v0.2.1.md).

### Fixed

- Rejected plus-prefixed bare-dot numbers such as `+.` instead of repairing
  them into invalid JSON.

### Added

- Added a public feature parity matrix comparing `jsonrepair-rs` with the
  JavaScript `jsonrepair` and Python `json-repair` packages.
- Added an LLM fallback parsing guide with copy-paste examples for
  `serde_json::Value`, typed `serde` deserialization, and strict mode.

### Changed

- Refreshed the README introduction to make the Rust-native positioning,
  trust signals, and known limits clear from the first screen.
- Excluded the README card image from the published crate package while keeping
  it rendered through a repository URL.

### Compatibility

- This release is source-compatible with `0.2.0`.
- The reader-to-writer API remains streaming-oriented at the IO boundary, but
  still buffers internally instead of performing constant-memory repair.
- Schema-guided repair is still not implemented; validate repaired data in the
  application layer.

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
