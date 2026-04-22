# jsonrepair-rs

[![CI](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/jsonrepair-rs.svg)](https://crates.io/crates/jsonrepair-rs)
[![Docs.rs](https://docs.rs/jsonrepair-rs/badge.svg)](https://docs.rs/jsonrepair-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![jsonrepair-rs: Repair malformed JSON into valid JSON](docs/assets/jsonrepair-rs-card.png)

Repair malformed JSON-like text and return valid JSON text.

`jsonrepair-rs` is a Rust library for cleaning up JSON commonly produced by
LLMs, copied from JavaScript/Python/MongoDB contexts, pasted from markdown, or
truncated in transit. It is a Rust port inspired by
[josdejong/jsonrepair](https://github.com/josdejong/jsonrepair).

This crate currently provides a library API only. It does not ship a CLI binary.

## Installation

```bash
cargo add jsonrepair-rs
```

Or add it manually:

```toml
[dependencies]
jsonrepair-rs = "0.1.1"
```

Minimum supported Rust version: 1.70.

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
```

Exports:

- `jsonrepair` repairs one input string.
- `JsonRepairError` contains `message`, `position`, `kind`, `line`, and `column`.
- `JsonRepairErrorKind` is a non-exhaustive enum for programmatic error handling.

The output is valid JSON when the function returns `Ok(...)`. When the input
cannot be repaired safely, the function returns an error instead of guessing.

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
- It does not provide streaming repair or a command-line binary.
- It is designed for practical repair, not for accepting arbitrary unsafe input
  as if it were trustworthy. Validate the repaired data according to your
  application's schema before using it.

## Examples

Runnable examples live in `examples/`:

```bash
cargo run --example repair_basic
cargo run --example repair_and_parse
```

`repair_basic` repairs and prints a malformed JSON-like string.
`repair_and_parse` repairs a string and then parses it with `serde_json`.

## Feature Flags

| Feature | Default | Notes |
| --- | --- | --- |
| `serde` | No | Enables optional `serde` and `serde_json` dependencies. The public API remains string-based. |

For most applications, add `serde_json` directly to your own `Cargo.toml` if you
want to parse the repaired string into typed data.

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

The latest crate published on crates.io is `0.1.1`.

To publish a new release, first bump the version in `Cargo.toml` and update any
version references in this README. Then run:

```bash
cargo test --all-targets
cargo doc --no-deps
cargo package
cargo publish --dry-run
```

If the dry run succeeds, publish with `cargo publish` and create a matching git
tag/release.

## License

MIT. See [LICENSE](LICENSE).
