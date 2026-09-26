# Changelog

## 0.2.5 - 2026-09-26

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.5.md`](docs/releases/v0.2.5.md).

### Added

- The optional `serde` feature now provides `jsonrepair_value_with_schema` for
  best-effort correction of tool arguments after JSON syntax repair. It handles
  numeric and boolean strings, unambiguous string enum casing, and non-null
  singleton values where an array is expected, including nested `properties`
  and `items`.

### Compatibility

- This is source-compatible with 0.2.4. Existing APIs, error types, and default
  repair behavior are unchanged.
- The new helper does not validate a full JSON Schema. Values it cannot
  correct safely remain unchanged; callers should validate before use.

## 0.2.4 - 2026-09-25

Release notes suitable for a GitHub Release are also available at
[`docs/releases/v0.2.4.md`](docs/releases/v0.2.4.md).

### Changed

- CI now checks the core library and CLI with Rust 1.70. The README clarifies
  that the optional `serde` feature also depends on upstream Rust support.
- The CLI scripting example preserves the command's original exit code after
  reporting usage, repair, or IO errors.

### Compatibility

- No runtime behavior, public API, or error type changed from 0.2.3.

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
- Reject isolated UTF-16 surrogate escapes instead of returning a string that
  downstream JSON parsers cannot decode. Valid surrogate pairs remain intact.

### Compatibility

- This is a source-compatible patch release. Public APIs and error types are
  unchanged.
- The fuzz validator now accepts JSON numbers and nesting depths supported by
  the repairer, avoiding false failures from `serde_json` default limits.

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
