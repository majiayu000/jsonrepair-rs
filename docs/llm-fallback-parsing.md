# LLM Fallback Parsing

Use `jsonrepair-rs` as an explicit fallback when normal JSON parsing fails.
Do not treat repair as validation: after repair, still deserialize into the
shape your application expects and validate required business rules.

Recommended flow:

1. Try `serde_json::from_str` first.
2. If strict JSON parsing succeeds, use that value unchanged.
3. If strict JSON parsing fails, call `jsonrepair-rs`.
4. Parse the repaired JSON with `serde_json`.
5. Validate the parsed value or typed struct before using it.
6. Report repair errors instead of silently replacing data with defaults.

## Parse To `serde_json::Value`

This pattern repairs only after strict JSON parsing fails. If repair fails, the
repair error is returned to the caller.

```rust
use jsonrepair_rs::jsonrepair;
use serde_json::Value;

fn parse_llm_value(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    match serde_json::from_str::<Value>(input) {
        Ok(value) => return Ok(value),
        Err(strict_error) => {
            let repaired = match jsonrepair(input) {
                Ok(repaired) => repaired,
                Err(repair_error) => {
                    eprintln!("strict JSON parse failed before repair: {strict_error}");
                    return Err(Box::new(repair_error));
                }
            };

            let value = serde_json::from_str::<Value>(&repaired)?;
            Ok(value)
        }
    }
}
```

Use this form when the downstream shape is dynamic, but still check that the
fields you need are present:

```rust
use serde_json::Value;

fn required_user(value: &Value) -> Result<&str, &'static str> {
    value
        .get("user")
        .and_then(Value::as_str)
        .ok_or("missing required `user` field")
}
```

## Parse To A Typed Struct

Enable the optional `serde` feature when you want repair-and-deserialize
helpers:

```toml
[dependencies]
jsonrepair-rs = { version = "0.2.1", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Then parse valid JSON first and fall back to repair only on failure:

```rust
use jsonrepair_rs::jsonrepair_parse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Profile {
    name: String,
    active: bool,
    skills: Vec<String>,
}

fn parse_profile(input: &str) -> Result<Profile, Box<dyn std::error::Error>> {
    match serde_json::from_str::<Profile>(input) {
        Ok(profile) => Ok(profile),
        Err(strict_error) => {
            let profile: Profile = jsonrepair_parse(input).map_err(|repair_error| {
                eprintln!("strict JSON parse failed before repair: {strict_error}");
                repair_error
            })?;

            if profile.name.trim().is_empty() {
                return Err("profile name must not be empty".into());
            }
            if profile.skills.is_empty() {
                return Err("profile must include at least one skill".into());
            }

            Ok(profile)
        }
    }
}
```

The typed parse step is useful because `serde` rejects missing fields and type
mismatches according to the struct definition. Application-specific checks still
belong in your code.

## Strict Mode

Use strict mode when callers are expected to submit valid JSON and you want an
error for repairable non-standard input:

```rust
use jsonrepair_rs::{jsonrepair_with_options, RepairOptions};

fn require_strict_json(input: &str) -> Result<String, jsonrepair_rs::JsonRepairError> {
    jsonrepair_with_options(input, RepairOptions::strict())
}
```

Strict mode is not schema validation. It only rejects input when the repaired
output would differ from the original text.

## Operational Notes

- Keep the original model output in logs or traces when repair fails.
- Surface repair failures to the caller or job record; do not replace them with
  `{}`, `[]`, or `null`.
- Treat repaired JSON as untrusted input until it passes your schema, typed
  deserialization, or business-rule validation.
- Prefer typed structs for stable LLM contracts. Use `serde_json::Value` when
  the schema is intentionally dynamic.
