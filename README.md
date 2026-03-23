# jsonrepair-rs

[![CI](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/majiayu000/jsonrepair-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Repair broken JSON in Rust. Fix quotes, commas, comments, trailing content, and 30+ other issues commonly found in LLM outputs.

Rust port of [josdejong/jsonrepair](https://github.com/josdejong/jsonrepair).

## Features

| Category | Examples |
|----------|----------|
| **Quote repair** | Single quotes → double, curly quotes, backticks, unquoted keys |
| **Comma repair** | Missing, trailing, and leading commas |
| **Comments** | `//`, `/* */`, `#` — stripped from output |
| **Python keywords** | `True` → `true`, `False` → `false`, `None` → `null` |
| **JS keywords** | `undefined` → `null`, `NaN` → `null`, `Infinity` → `null` |
| **Markdown fences** | `` ```json ... ``` `` — extracted and repaired |
| **Truncated JSON** | Auto-closes unclosed brackets, braces, and strings |
| **Number repair** | Leading zeros, trailing dots (`2.` → `2.0`), truncated exponents |
| **String repair** | Concatenation (`"a" + "b"`), invalid escapes, unescaped control chars |
| **JSONP** | `callback({...})` → `{...}` |
| **MongoDB** | `ObjectId("...")` → `"..."`, `NumberLong("...")` → `"..."` |
| **NDJSON** | Newline-delimited JSON → JSON array |
| **Ellipsis** | `[1, 2, ...]` → `[1, 2]` |
| **Misc** | BOM stripping, special whitespace, trailing semicolons |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
jsonrepair-rs = "0.1"
```

```rust
use jsonrepair_rs::jsonrepair;

// Fix single quotes
let result = jsonrepair("{'name': 'John'}").unwrap();
assert_eq!(result, r#"{"name":"John"}"#);

// Fix trailing commas
let result = jsonrepair(r#"{"a": 1, "b": 2,}"#).unwrap();
assert_eq!(result, r#"{"a":1,"b":2}"#);

// Strip markdown fences
let result = jsonrepair("```json\n{\"a\": 1}\n```").unwrap();
assert_eq!(result, r#"{"a":1}"#);

// Convert Python keywords
let result = jsonrepair("{\"flag\": True, \"value\": None}").unwrap();
assert_eq!(result, r#"{"flag":true,"value":null}"#);

// Handle LLM output with comments
let result = jsonrepair(r#"{
    // user info
    name: "Alice",
    age: 30,
}"#).unwrap();
assert_eq!(result, r#"{"name":"Alice","age":30}"#);
```

## Error handling

```rust
use jsonrepair_rs::{jsonrepair, JsonRepairError};

match jsonrepair("not repairable at all") {
    Ok(json) => println!("Repaired: {json}"),
    Err(e) => eprintln!("Error at position {}: {}", e.position, e.message),
}
```

## Acknowledgments

This is a Rust port of [jsonrepair](https://github.com/josdejong/jsonrepair) by Jos de Jong.
