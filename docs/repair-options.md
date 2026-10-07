# Repair Options

`jsonrepair-rs` defaults to forgiving repair behavior for LLM output, copied JS
objects, markdown-fenced JSON, and other JSON-like input.

Use `RepairOptions` when the caller needs an explicit policy.

## Default Policy

```rust
use jsonrepair_rs::{jsonrepair_with_options, RepairOptions};

let repaired = jsonrepair_with_options(
    "{name: 'Ada', active: True}",
    RepairOptions::default(),
)?;
assert_eq!(repaired, r#"{"name": "Ada", "active": true}"#);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`RepairOptions::default()` is equivalent to `jsonrepair(input)`.

## Strict Mode

Strict mode returns valid JSON unchanged and rejects input that would require
repair:

```rust
use jsonrepair_rs::{jsonrepair_with_options, JsonRepairErrorKind, RepairOptions};

let valid = r#"{"name": "Ada", "active": true}"#;
assert_eq!(
    jsonrepair_with_options(valid, RepairOptions::strict())?,
    valid
);

let err = jsonrepair_with_options("{name: 'Ada'}", RepairOptions::strict())
    .unwrap_err();
assert_eq!(err.kind, JsonRepairErrorKind::StrictModeViolation);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use strict mode when accepting user input that should already be JSON, or when a
caller needs to distinguish "valid as submitted" from "repairable but
non-standard."

## Comment Markers and Unquoted Escapes

Two independent options address content in malformed tool-call arguments:

| Builder method | Default | Effect when enabled |
| --- | --- | --- |
| `with_preserve_comment_markers(true)` | `false` | Disable comment stripping throughout the input and allow slashes in unquoted strings, preserving `#`, `//` and `/* ... */` as content. |
| `with_decode_unquoted_escapes(true)` | `false` | Interpret complete JSON escapes in unquoted strings, including keys. |

For example, preserve a Markdown heading while keeping backslashes literal:

```rust
use jsonrepair_rs::{jsonrepair_with_options, RepairOptions};

let options = RepairOptions::new().with_preserve_comment_markers(true);
assert_eq!(
    jsonrepair_with_options(r##"{"plan": # Title\nSub"}"##, options)?,
    r##"{"plan": "# Title\\nSub"}"##,
);
assert_eq!(
    jsonrepair_with_options(r#"{"path": C:\new\table.txt}"#, options)?,
    r#"{"path": "C:\\new\\table.txt"}"#,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Enable escape decoding separately when the caller intends literal `\n` input
to become a newline in the parsed value:

```rust
use jsonrepair_rs::{jsonrepair_with_options, RepairOptions};

let options = RepairOptions::new()
    .with_preserve_comment_markers(true)
    .with_decode_unquoted_escapes(true);
assert_eq!(
    jsonrepair_with_options(r##"{"plan": # Title\nSub"}"##, options)?,
    r##"{"plan": "# Title\nSub"}"##,
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Escape decoding recognizes `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t` and
complete `\uXXXX` escapes. Invalid or incomplete escapes stay literal; lone or
mismatched Unicode surrogates return `InvalidUnicode`, as in quoted strings.
Escaped quotes inside an unquoted token become content rather than terminating
the token. Properly quoted strings retain their existing behavior.

**Escape decoding also interprets the `\n` and `\t` in `C:\new\table.txt`.**
Leave it disabled for literal paths. The parser cannot determine whether an
unquoted backslash is intended as data or an escape.

Comment preservation disables comment syntax globally, rather than guessing
which comment was intended as string content. It can change how regex-like
unquoted values are repaired because slashes become string content. The options
do not recover arbitrary malformed arguments or enable actual multiline
unquoted strings: a real line break still ends an unquoted token. Strict mode
continues to reject input requiring repair.

## Supported Helpers

Options are available on the string, writer, reader-to-writer, and serde helper
surfaces:

```rust
jsonrepair_with_options(input, options)
jsonrepair_to_writer_with_options(input, writer, options)
jsonrepair_reader_to_writer_with_options(reader, writer, options)
jsonrepair_value_with_options(input, options)
jsonrepair_parse_with_options::<T>(input, options)
```

The serde helpers require the `serde` feature.

## Future Policy Toggles

Further policy fields can be added without changing `jsonrepair(input)` callers.
Candidate toggles include:

- Python and JavaScript keywords
- markdown fences
- JSONP wrappers
- NDJSON aggregation
- non-finite numbers

Those toggles should be added only when each policy has representative tests and
clear default compatibility behavior.
