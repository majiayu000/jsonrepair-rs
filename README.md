# jsonrepair-rs

[![CI](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/jsonrepair-rs.svg)](https://crates.io/crates/jsonrepair-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Repair broken JSON in Rust.

`jsonrepair-rs` takes malformed JSON-like text (often from LLM output) and returns valid JSON text.

Rust port of [josdejong/jsonrepair](https://github.com/josdejong/jsonrepair).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jsonrepair-rs = "0.1"
```

## Quick Start

This crate is a **library crate** (no built-in CLI binary).

```rust
use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broken = r#"{name: 'Alice', active: True, skills: ['Rust',],}"#;
    let repaired = jsonrepair(broken)?;

    // repaired is always a valid JSON string when Ok(...)
    println!("{repaired}");
    Ok(())
}
```

Output:

```json
{"name": "Alice", "active": true, "skills": ["Rust"]}
```

## Examples

This repository includes runnable examples in `examples/`:

```bash
cargo run --example repair_basic
cargo run --example repair_and_parse
```

- `repair_basic`: repairs a malformed JSON-like string and prints the result.
- `repair_and_parse`: repairs input, parses with `serde_json`, and validates key fields.

## API

```rust
pub fn jsonrepair(input: &str) -> Result<String, JsonRepairError>
```

- Input: malformed JSON-like text.
- Output: repaired JSON string.
- On failure: returns `JsonRepairError` with `kind`, `position`, `line`, and `column`.

If you need a typed value, parse the repaired string with `serde_json`:

```rust
use jsonrepair_rs::jsonrepair;

let repaired = jsonrepair("{a:1, b:2,}").unwrap();
let value: serde_json::Value = serde_json::from_str(&repaired).unwrap();
assert_eq!(value["a"], 1);
```

## Common Repairs

| Category | Examples |
| --- | --- |
| Quote repair | single quotes, curly quotes, backticks, unquoted keys |
| Comma repair | missing commas, trailing commas, leading commas |
| Comments | `//`, `/* */`, `#` comments are removed |
| Python keywords | `True` → `true`, `False` → `false`, `None` → `null` |
| JS keywords | `undefined`/`NaN`/`Infinity` → `null` |
| Markdown fences | extracts content from fenced blocks like `````json ... ````` |
| Truncated JSON | auto-closes missing `]`, `}`, and string terminators |
| Number fixes | leading zeros, trailing dots (`2.` → `2.0`), truncated exponents |
| String fixes | concatenation (`"a" + "b"`), invalid escapes, control chars |
| JSONP | `callback({...})` → `{...}` |
| MongoDB wrappers | `ObjectId("...")`, `NumberLong("...")` |
| NDJSON | newline-delimited JSON converted to a JSON array |
| Ellipsis | `[1, 2, ...]` → `[1, 2]` |

## Error Handling

```rust
use jsonrepair_rs::{jsonrepair, JsonRepairErrorKind};

match jsonrepair("not repairable at all") {
    Ok(json) => println!("Repaired: {json}"),
    Err(e) => {
        eprintln!("kind={:?}, at {}:{} (pos={}): {}", e.kind, e.line, e.column, e.position, e.message);

        if matches!(e.kind, JsonRepairErrorKind::MaxDepthExceeded) {
            eprintln!("input nesting depth exceeded the internal limit");
        }
    }
}
```

## Notes

- Maximum supported nesting depth is 512.
- When no safe repair is possible, the function returns an error instead of guessing.

## Development

```bash
# Build and test
cargo build
cargo test

# CI-equivalent checks
RUSTFLAGS="-Dwarnings" cargo check --all-targets
cargo fmt --all -- --check
cargo doc --no-deps
```

## Pre-commit

```bash
# Install pre-commit (using uv)
uv tool install pre-commit

# Install commit + push hooks
pre-commit install
pre-commit install --hook-type pre-push

# Run commit hooks manually
pre-commit run --all-files

# Run push hooks manually
pre-commit run --all-files --hook-stage pre-push
```

## Benchmarks

```bash
cargo bench
```

### Optimization Round Automation

Use this script to run one full optimization validation round with automatic gating:

```bash
# Compare against an existing Criterion baseline
scripts/opt_round.sh --baseline current_bec2481
```

Default gate policy:
- fail if any benchmark is statistically regressed (mean CI lower bound > 0)
- allow unchanged benchmarks

Strict mode (all benchmarks must improve):

```bash
scripts/opt_round.sh --baseline current_bec2481 --require-all-improved
```

The script writes a per-round report to:
- `.omx/reports/opt-round-<timestamp>.md`

## Acknowledgments

This is a Rust port of [jsonrepair](https://github.com/josdejong/jsonrepair) by Jos de Jong.
