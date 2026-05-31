# jsonrepair-rs

[![CI](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/jsonrepair-rs.svg)](https://crates.io/crates/jsonrepair-rs)
[![Docs.rs](https://docs.rs/jsonrepair-rs/badge.svg)](https://docs.rs/jsonrepair-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![jsonrepair-rs: Repair malformed JSON into valid JSON](docs/assets/jsonrepair-rs-card.png)

Repair malformed JSON-like text and return valid JSON text.

`jsonrepair-rs` is a Rust-native repair library for applications that receive
structured JSON from LLMs, copied JavaScript/Python/MongoDB snippets, markdown
code fences, logs, or truncated transport payloads. It is a Rust port inspired
by [josdejong/jsonrepair](https://github.com/josdejong/jsonrepair), with broad
compatibility goals against the JavaScript `jsonrepair` and Python
`json-repair` ecosystems.

## Why This Crate

- Rust library and CLI surfaces in one package.
- No default dependencies; optional `serde` helpers are behind a feature flag.
- Repair behavior covers quotes, commas, comments, markdown fences, truncated
  JSON, JSONP, MongoDB wrappers, NDJSON, non-standard keywords, and more.
- Strict mode lets callers reject input that would require repair.
- Errors include kind, position, line, and column so callers can report failure
  instead of silently dropping malformed data.

## Trust And Limits

- Repair behavior is covered by unit tests, CLI tests, writer/reader tests,
  strict-mode tests, serde tests, fuzz harnesses, and upstream-style parity
  fixtures.
- The output is valid JSON when an API returns `Ok(...)`; unrecoverable input
  returns a typed error instead of a guessed repair.
- The reader-to-writer API is streaming-oriented at the IO boundary, but the
  current parser still buffers internally. See [`docs/streaming-api.md`](docs/streaming-api.md).
- This crate does not do schema-guided repair or schema validation. Validate
  repaired data against your application's schema before using it.
- See [`FEATURE_PARITY.md`](FEATURE_PARITY.md) for a side-by-side comparison
  with JS/Python repair libraries and [`docs/competitor-comparison.md`](docs/competitor-comparison.md)
  for local comparison tooling.

## Installation

```bash
cargo add jsonrepair-rs
```

Or add it manually:

```toml
[dependencies]
jsonrepair-rs = "0.2.0"
```

Minimum supported Rust version: 1.70.

## Command Line

Install the binary with Cargo:

```bash
cargo install jsonrepair-rs
```

Repair stdin to stdout:

```bash
printf "{name: 'Ada', active: True}" | jsonrepair
```

Repair a file and write the result to another file:

```bash
jsonrepair broken.json --output repaired.json
```

## Quick Start

```rust
use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broken = r#"{name: 'Alice', active: True, skills: ['Rust',],}"#;
    let repaired = jsonrepair(broken)?;

    assert_eq!(
        repaired,
        r#"{"name": "Alice", "active": true, "skills": ["Rust"]}"#
    );

    println!("{repaired}");
    Ok(())
}
```

`jsonrepair` returns a JSON string. If you need typed data, parse the repaired
string with `serde_json` in your application:

```rust
use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repaired = jsonrepair("{user: 'Ada', admin: False, attempts: [1,2,],}")?;
    let value: serde_json::Value = serde_json::from_str(&repaired)?;

    assert_eq!(value["user"], "Ada");
    assert_eq!(value["admin"], false);
    assert_eq!(value["attempts"][1], 2);

    Ok(())
}
```

## API

```rust
pub fn jsonrepair(input: &str) -> Result<String, JsonRepairError>
pub fn jsonrepair_with_options(input: &str, options: RepairOptions) -> Result<String, JsonRepairError>
pub fn jsonrepair_to_writer<W>(input: &str, writer: &mut W) -> Result<(), JsonRepairWriteError>
pub fn jsonrepair_to_writer_with_options<W>(input: &str, writer: &mut W, options: RepairOptions) -> Result<(), JsonRepairWriteError>
pub fn jsonrepair_reader_to_writer<R, W>(reader: R, writer: &mut W) -> Result<(), JsonRepairStreamError>
pub fn jsonrepair_reader_to_writer_with_options<R, W>(reader: R, writer: &mut W, options: RepairOptions) -> Result<(), JsonRepairStreamError>
pub fn jsonrepair_value(input: &str) -> Result<serde_json::Value, JsonRepairParseError>
pub fn jsonrepair_value_with_options(input: &str, options: RepairOptions) -> Result<serde_json::Value, JsonRepairParseError>
pub fn jsonrepair_parse<T>(input: &str) -> Result<T, JsonRepairParseError>
pub fn jsonrepair_parse_with_options<T>(input: &str, options: RepairOptions) -> Result<T, JsonRepairParseError>
```

| API | Feature | Returns | Failure modes |
| --- | --- | --- | --- |
| `jsonrepair(input)` | default | repaired JSON `String` | `JsonRepairError` when input cannot be repaired safely |
| `jsonrepair_with_options(input, options)` | default | repaired JSON `String` with explicit policy | `JsonRepairError`, including `StrictModeViolation` in strict mode |
| `jsonrepair_to_writer(input, writer)` | default | writes repaired JSON to `std::io::Write` | `JsonRepairWriteError::Repair` or `JsonRepairWriteError::Write` |
| `jsonrepair_to_writer_with_options(input, writer, options)` | default | writes with explicit policy | `JsonRepairWriteError::Repair` or `JsonRepairWriteError::Write` |
| `jsonrepair_reader_to_writer(reader, writer)` | default | reads from `std::io::Read`, writes to `std::io::Write` | `JsonRepairStreamError::Read`, `Repair`, or `Write` |
| `jsonrepair_reader_to_writer_with_options(reader, writer, options)` | default | streams with explicit policy | `JsonRepairStreamError::Read`, `Repair`, or `Write` |
| `jsonrepair_value(input)` | `serde` | repaired `serde_json::Value` | `JsonRepairParseError::Repair` or `JsonRepairParseError::Parse` |
| `jsonrepair_value_with_options(input, options)` | `serde` | repaired `serde_json::Value` with explicit policy | `JsonRepairParseError::Repair` or `JsonRepairParseError::Parse` |
| `jsonrepair_parse<T>(input)` | `serde` | repaired and deserialized `T` | `JsonRepairParseError::Repair` or `JsonRepairParseError::Parse` |
| `jsonrepair_parse_with_options<T>(input, options)` | `serde` | repaired and deserialized `T` with explicit policy | `JsonRepairParseError::Repair` or `JsonRepairParseError::Parse` |

Supporting types:

- `JsonRepairError` contains `message`, `position`, `kind`, `line`, and
  `column`.
- `JsonRepairErrorKind` is a non-exhaustive enum for programmatic repair-error
  handling.
- `JsonRepairWriteError`, `JsonRepairStreamError`, and `JsonRepairParseError`
  distinguish repair failures from IO or `serde_json` parse failures.

The output is valid JSON when the function returns `Ok(...)`. When the input
cannot be repaired safely, the function returns an error instead of guessing.
Use [`RepairOptions::strict`](docs/repair-options.md) when callers want valid
JSON pass-through and an error for repairable non-standard input.

The reader-to-writer API is streaming-oriented at the IO boundary, but the
current parser still buffers complete input and repaired output internally. See
[`docs/streaming-api.md`](docs/streaming-api.md) for the design and memory
tradeoffs.

Repair a file into another file:

```rust,no_run
use std::fs::File;
use jsonrepair_rs::jsonrepair_reader_to_writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = File::open("broken.json")?;
    let mut output = File::create("repaired.json")?;

    jsonrepair_reader_to_writer(&mut input, &mut output)?;
    Ok(())
}
```

## What It Repairs

| Input style | Example repair |
| --- | --- |
| Single, curly, and backtick quotes | `{'a': 'b'}` -> `{"a": "b"}` |
| Unquoted object keys | `{name: "Ada"}` -> `{"name": "Ada"}` |
| Missing commas | `[1 2 3]` -> `[1, 2, 3]` |
| Leading or trailing commas | `[1,2,]` -> `[1,2]` |
| Missing colons | `{"a" 1}` -> `{"a": 1}` |
| Missing object values | `{"a":}` -> `{"a":null}` |
| JavaScript, Python, and case variants | `True`, `False`, `None`, `undefined`, `NaN`, `Infinity` |
| Signed non-finite values | `-Infinity`, `+NaN` -> `null` |
| Comments | `//`, `/* ... */`, and `#` comments are removed |
| Markdown code fences | ````json ... ```` wrappers are stripped |
| Truncated JSON | missing brackets, braces, strings, and exponents are completed |
| Redundant closing brackets | `{"a":1}}` -> `{"a":1}` |
| Number fixes | `.5`, `+.5`, `2.`, `2e`, `2e+` |
| Invalid numbers as strings | `0.0.1` -> `"0.0.1"` |
| String fixes | missing quotes, invalid escapes, unescaped control chars |
| String concatenation | `"a" + "b"` -> `"ab"` |
| JSONP | `callback({"a":1});` -> `{"a":1}` |
| MongoDB wrappers | `ObjectId("...")`, `NumberLong("...")`, `NumberInt(...)` |
| NDJSON / root value lists | newline-delimited values become an array |
| URLs and regex-like tokens | unquoted URL and regex-like text become strings |
| Ellipsis placeholders | `[1, 2, ...]` -> `[1, 2]` |
| BOM and special whitespace | normalized outside strings |

The test suite contains many edge cases for these categories in
`tests/repair_tests.rs`.

## Error Handling

```rust
use jsonrepair_rs::{jsonrepair, JsonRepairErrorKind};

match jsonrepair("") {
    Ok(json) => println!("repaired: {json}"),
    Err(err) => {
        eprintln!(
            "kind={:?}, line={}, column={}, position={}: {}",
            err.kind, err.line, err.column, err.position, err.message
        );

        if matches!(err.kind, JsonRepairErrorKind::UnexpectedEnd) {
            eprintln!("input ended before a repairable JSON value was found");
        }
    }
}
```

Error kinds are marked `#[non_exhaustive]`; include a fallback arm when matching
them outside this crate.

## Limits And Behavior

- Maximum supported nesting depth is 512.
- The crate preserves much of the original whitespace where possible.
- It returns a repaired JSON string, not a `serde_json::Value`.
- The `jsonrepair_reader_to_writer` API supports reader-to-writer workflows,
  but `0.2.0` still buffers internally instead of performing constant-memory
  repair.
- It is designed for practical repair, not for accepting arbitrary unsafe input
  as if it were trustworthy. Validate the repaired data according to your
  application's schema before using it.

## Examples

Runnable examples live in `examples/`:

```bash
cargo run --example repair_basic
cargo run --example repair_and_parse
cargo run --example llm_output
cargo run --features serde --example llm_typed_parse
```

`repair_basic` repairs and prints a malformed JSON-like string.
`repair_and_parse` repairs a string and then parses it with `serde_json`.
`llm_output` repairs an LLM-style response that has prose around a JSON-like
object. The prose is preserved as strings in a valid JSON array, so callers can
inspect or extract the object they need.
`llm_typed_parse` repairs a markdown-fenced JSON object and deserializes it into
a typed struct with the `serde` feature enabled.
For production-style fallback patterns, see
[`docs/llm-fallback-parsing.md`](docs/llm-fallback-parsing.md).

For a shell pipeline, pass fenced JSON through the CLI:

````bash
printf '```json
{name: "Ada", active: True, skills: ["rust",],}
```' | jsonrepair
````

This prints valid JSON for the fenced object:

```json
{"name": "Ada", "active": true, "skills": ["rust"]}
```

## Feature Flags

| Feature | Default | Notes |
| --- | --- | --- |
| `serde` | No | Enables optional `serde` and `serde_json` dependencies plus repair-and-parse helpers. |

For most applications, add `serde_json` directly to your own `Cargo.toml` if you
want to parse the repaired string into typed data.

With the `serde` feature enabled, the crate also provides convenience helpers
for repair-and-parse workflows:

```rust
use jsonrepair_rs::jsonrepair_value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = jsonrepair_value("{name: 'Ada', active: True}")?;

    assert_eq!(value["name"], "Ada");
    assert_eq!(value["active"], true);
    Ok(())
}
```

## Development

```bash
cargo build
cargo test --all-targets
```

CI-equivalent local checks:

```bash
RUSTFLAGS="-Dwarnings" cargo check --all-targets
cargo fmt --all -- --check
cargo test --all-targets
cargo doc --no-deps
```

The GitHub Actions workflow runs check, formatting, tests, and docs on `main`
pushes and pull requests.

## Pre-commit Hooks

```bash
uv tool install pre-commit

pre-commit install
pre-commit install --hook-type pre-push

pre-commit run --all-files
pre-commit run --all-files --hook-stage pre-push
```

## Benchmarks

Run Criterion benchmarks:

```bash
cargo bench
```

See [`docs/benchmarking.md`](docs/benchmarking.md) for CLI throughput reports,
optional Rust competitor benchmarks, and allocation profiling commands.

Current benchmark groups cover:

- valid small JSON
- broken small JSON
- valid 1k-item JSON
- broken 1k-item JSON
- 100-level nesting
- 100 comments
- 200 string escapes

For optimization work, use the benchmark gate script with an existing Criterion
baseline:

```bash
scripts/opt_round.sh --baseline current_bec2481
```

The script runs:

1. `cargo fmt`
2. `cargo check --all-targets`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `cargo test --all-targets`
5. Criterion benchmarks against the selected baseline
6. Stable regression detection with optional reruns and a control self-check

Useful options:

```bash
scripts/opt_round.sh --baseline current_bec2481 --eps 0.01
scripts/opt_round.sh --baseline current_bec2481 --require-all-improved
scripts/opt_round.sh --baseline current_bec2481 --reruns-on-regression 4
scripts/opt_round.sh --baseline current_bec2481 --skip-checks
```

Reports are written to `.omx/reports/opt-round-<timestamp>.md` by default. If
the benchmark environment is too noisy to trust, the script exits inconclusive
instead of reporting a false regression.

## Release Status

The latest crate published on crates.io is `0.2.0`.

To publish a new release, first bump the version in `Cargo.toml` and update any
version references in this README. Then follow
[`docs/release-checklist.md`](docs/release-checklist.md). The minimum local
gate is:

```bash
RUSTFLAGS="-Dwarnings" cargo check --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
cargo package
cargo publish --dry-run
```

If the dry run succeeds, publish with `cargo publish` and create a matching git
tag/release.

See [`docs/ecosystem-evaluations.md`](docs/ecosystem-evaluations.md) for the
planned order of binary distribution, WASM/npm bindings, and Python bindings.

## License

MIT. See [LICENSE](LICENSE).
