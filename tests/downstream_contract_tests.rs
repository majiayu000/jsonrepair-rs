//! Synthetic, dependency-free cases shaped by public downstream call sites.
//! See docs/downstream-compatibility.md for provenance and scope. These are
//! crate-level contracts, not tests of downstream candidate selection or tools.

use std::io::{self, Cursor, Read, Write};

use jsonrepair_rs::{
    jsonrepair, jsonrepair_reader_to_writer, jsonrepair_to_writer, jsonrepair_with_options,
    JsonRepairErrorKind, JsonRepairStreamError, RepairOptions,
};
use serde_json::{json, Value};

fn assert_repair(input: &str, expected: Value) {
    let repaired = jsonrepair(input).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&repaired).unwrap(), expected);
    assert_eq!(jsonrepair(&repaired).unwrap(), repaired);
    assert_eq!(
        jsonrepair_with_options(&repaired, RepairOptions::strict()).unwrap(),
        repaired
    );

    let mut written = Vec::new();
    jsonrepair_to_writer(input, &mut written).unwrap();
    assert_eq!(written, repaired.as_bytes());

    // Include splits inside UTF-8, escapes, comment markers and delimiters.
    for chunk_size in 1..=5 {
        let mut reader = ChunkedReader {
            input: Cursor::new(input.as_bytes()),
            chunk_size,
        };
        let mut streamed = Vec::new();
        jsonrepair_reader_to_writer(&mut reader, &mut streamed).unwrap();
        assert_eq!(streamed, repaired.as_bytes(), "chunk size {chunk_size}");
    }

    #[cfg(feature = "serde")]
    assert_eq!(jsonrepair_rs::jsonrepair_value(input).unwrap(), expected);
}

struct ChunkedReader<'a> {
    input: Cursor<&'a [u8]>,
    chunk_size: usize,
}

impl Read for ChunkedReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let len = buffer.len().min(self.chunk_size);
        self.input.read(&mut buffer[..len])
    }
}

#[test]
fn cli_candidate_payloads() {
    for (input, expected) in [
        (
            "{recipient: 'demo', content: 'hello',}",
            json!({"recipient": "demo", "content": "hello"}),
        ),
        (
            r#"{"content":"say "hello" now"}"#,
            json!({"content": "say \"hello\" now"}),
        ),
        (
            "{enabled: True, optional: None, tags: ['rust',]}",
            json!({"enabled": true, "optional": null, "tags": ["rust"]}),
        ),
    ] {
        assert_repair(input, expected);
    }
}

#[test]
fn completed_tool_arguments_with_truncated_syntax() {
    for (input, expected) in [
        (r#"{"path":"src/demo.rs"#, json!({"path": "src/demo.rs"})),
        (
            r#"{"files":["a.rs","b.rs"#,
            json!({"files": ["a.rs", "b.rs"]}),
        ),
        (
            r#"{"output":{"status":"done"},"assistant_message":null,}"#,
            json!({"output": {"status": "done"}, "assistant_message": null}),
        ),
        (
            r#"{"query":"中文 🦀","limit":3"#,
            json!({"query": "中文 🦀", "limit": 3}),
        ),
    ] {
        assert_repair(input, expected);
    }
}

#[test]
fn quoted_content_and_urls_keep_comment_markers() {
    for (input, expected) in [
        (
            r##"{"content":"# Release notes\n// example\n/* text */"}"##,
            json!({"content": "# Release notes\n// example\n/* text */"}),
        ),
        (
            r#"{"url":https://example.org/path#section}"#,
            json!({"url": "https://example.org/path#section"}),
        ),
        (
            r#"{"path":"//server/share","pattern":"/*literal*/"}"#,
            json!({"path": "//server/share", "pattern": "/*literal*/"}),
        ),
    ] {
        assert_repair(input, expected);
    }
}

#[test]
fn default_comment_policy_is_not_a_content_preserving_tool_profile() {
    // Historical behavior: a missing opening quote makes these comments, not
    // recoverable content. Do not silently change this policy in a patch release.
    for input in [
        r##"{"content": # Release notes"}"##,
        r##"{"content": // Release notes"}"##,
        r##"{"content": /* Release notes */}"##,
    ] {
        assert_repair(input, json!({"content": null}));
        let err = jsonrepair_with_options(input, RepairOptions::strict()).unwrap_err();
        assert_eq!(err.kind, JsonRepairErrorKind::StrictModeViolation);
    }
    assert_repair(
        "{\n# config\nname:'demo', // line\nactive:True /* block */\n}",
        json!({"name": "demo", "active": true}),
    );
}

#[test]
fn read_failure_after_valid_prefix_writes_nothing() {
    struct FailAfterPrefix(bool);
    impl Read for FailAfterPrefix {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.0 && !buffer.is_empty() {
                self.0 = true;
                buffer[0] = b'0';
                Ok(1)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "synthetic read failure",
                ))
            }
        }
    }
    let mut output = b"existing".to_vec();
    let error = jsonrepair_reader_to_writer(FailAfterPrefix(false), &mut output).unwrap_err();
    assert!(matches!(error, JsonRepairStreamError::Read(_)));
    assert_eq!(output, b"existing");
}

#[test]
fn write_failure_can_leave_partial_output() {
    #[derive(Default)]
    struct FailAfterByte(Vec<u8>);
    impl Write for FailAfterByte {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if self.0.is_empty() && !buffer.is_empty() {
                self.0.push(buffer[0]);
                Ok(1)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "synthetic write failure",
                ))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut output = FailAfterByte::default();
    let error = jsonrepair_reader_to_writer(Cursor::new("{key:1}"), &mut output).unwrap_err();
    assert!(matches!(error, JsonRepairStreamError::Write(_)));
    assert_eq!(output.0, b"{");
}

#[cfg(feature = "serde")]
#[test]
fn schema_correction_does_not_validate_tool_arguments() {
    let schema = json!({
        "type": "object",
        "required": ["missing"],
        "additionalProperties": false,
        "properties": {
            "count": {"type": "integer", "minimum": 10},
            "mode": {"type": "string", "enum": ["SAFE", "safe"]},
            "items": {"type": "array", "minItems": 1},
            "via_ref": {"$ref": "#/$defs/count"},
            "union": {"type": ["integer", "null"]}
        },
        "$defs": {"count": {"type": "integer"}}
    });
    let input =
        r#"{"count":"2","mode":"Safe","items":null,"via_ref":"3","union":"4","extra":true}"#;
    let result = jsonrepair_rs::jsonrepair_value_with_schema(input, &schema).unwrap();
    assert_eq!(
        result,
        json!({
            "count": 2, "mode": "Safe", "items": null,
            "via_ref": "3", "union": "4", "extra": true
        })
    );
    // Ok is only a successful repair/correction: missing required properties,
    // bounds, additional properties, $ref and union types are not validated.
}
