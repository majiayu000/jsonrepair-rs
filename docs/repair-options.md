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

The first options API intentionally exposes only strict mode. Future policy
fields can be added without breaking `jsonrepair(input)` callers. Candidate
toggles include:

- comments
- Python and JavaScript keywords
- markdown fences
- JSONP wrappers
- NDJSON aggregation
- non-finite numbers

Those toggles should be added only when each policy has representative tests and
clear default compatibility behavior.
