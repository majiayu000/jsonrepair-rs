//! Boundary and generated-input checks for the independent content policies.
//! Run on both native Windows and Unix; no downstream application is simulated.

use std::io::{self, Read};

use jsonrepair_rs::{
    jsonrepair_reader_to_writer_with_options, jsonrepair_to_writer_with_options,
    jsonrepair_with_options, JsonRepairErrorKind, JsonRepairStreamError, JsonRepairWriteError,
    RepairOptions,
};
use serde_json::{json, Value};

struct Chunked<'a> {
    bytes: &'a [u8],
    size: usize,
}

impl Read for Chunked<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.bytes.len().min(buffer.len()).min(self.size);
        buffer[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        Ok(count)
    }
}

fn options(preserve: bool, decode: bool) -> RepairOptions {
    RepairOptions::new()
        .with_preserve_comment_markers(preserve)
        .with_decode_unquoted_escapes(decode)
}

fn check_result(input: &str, policy: RepairOptions, chunks: &[usize]) -> Option<Value> {
    let result = jsonrepair_with_options(input, policy);
    let mut written = Vec::new();
    let write_result = jsonrepair_to_writer_with_options(input, &mut written, policy);
    let value = match &result {
        Ok(repaired) => {
            let value: Value = serde_json::from_str(repaired).unwrap_or_else(|error| {
                panic!("invalid JSON for {input:?}, {policy:?}: {repaired:?}: {error}")
            });
            assert_eq!(
                jsonrepair_with_options(repaired, policy).unwrap(),
                *repaired,
                "idempotence for {input:?}, {policy:?}"
            );
            assert_eq!(
                jsonrepair_with_options(repaired, policy.with_strict(true)).unwrap(),
                *repaired
            );
            write_result.unwrap();
            assert_eq!(written, repaired.as_bytes());
            Some(value)
        }
        Err(expected) => {
            match write_result {
                Err(JsonRepairWriteError::Repair(actual)) => assert_eq!(actual, *expected),
                other => panic!("writer mismatch for {input:?}: {other:?}"),
            }
            assert!(written.is_empty());
            None
        }
    };
    for &size in chunks {
        let mut streamed = Vec::new();
        let actual = jsonrepair_reader_to_writer_with_options(
            Chunked {
                bytes: input.as_bytes(),
                size,
            },
            &mut streamed,
            policy,
        );
        match (&result, actual) {
            (Ok(repaired), Ok(())) => assert_eq!(streamed, repaired.as_bytes()),
            (Err(expected), Err(JsonRepairStreamError::Repair(actual))) => {
                assert_eq!(actual, *expected);
                assert!(streamed.is_empty());
            }
            (expected, actual) => panic!("reader mismatch for {input:?}: {expected:?}, {actual:?}"),
        }
    }
    value
}

#[test]
fn literal_windows_paths_survive_in_values_arrays_and_keys() {
    for path in [
        r"C:\new\table.txt",
        r"C:\Users\name\file.txt",
        r"C:\Program Files\demo\config.json",
        r"C:\目录\报告🦀.txt",
        r"\\server\share\notes.txt",
        r"\\?\C:\new\table.txt",
        r"\\?\UNC\server\share\folder\",
        r".\relative\folder\",
        r"//server/share/name#hash.txt",
    ] {
        let policy = options(true, false);
        for newline in ["\n", "\r\n"] {
            let input = format!("{{{newline}  \"path\": {path}{newline}}}");
            assert_eq!(
                check_result(&input, policy, &[1, 2, 3, 7]),
                Some(json!({"path": path}))
            );
        }
        assert_eq!(
            check_result(&format!("[{path}]"), policy, &[1, 4]),
            Some(json!([path]))
        );
        // Drive-letter colons are object-key separators, so test UNC keys only.
        if !path.contains(':') {
            assert_eq!(
                check_result(&format!("{{{path}: 1}}"), policy, &[1, 3]),
                Some(json!({path: 1}))
            );
        }
    }
}

#[test]
fn escaped_delimiters_do_not_end_unquoted_tokens() {
    let policy = options(true, true);
    for encoded in [
        r"prefix\u002cend",
        r"prefix\u007bend",
        r"prefix\u005bend",
        r"prefix\u002bend",
        r"prefix\u003bend",
        r"prefix\u0029end",
        r"prefix\u0000end",
        r"prefix\u0020end",
        r"prefix\uFEFFend",
        r"prefix\uD83D\uDE00end",
        r#"prefix\"end"#,
        r"prefix\/end",
        r"prefix\\end",
        r"prefix\nend",
        r"prefix\tend",
    ] {
        let expected: String = serde_json::from_str(&format!("\"{encoded}\"")).unwrap();
        for input in [
            format!("{{\"text\": {encoded}}}"),
            format!("{{\"text\": {encoded}, \"next\": 1}}"),
            format!("callback({encoded})"),
            format!("[{encoded}]"),
        ] {
            let value = check_result(&input, policy, &[1, 2, 5]).unwrap();
            match value {
                Value::Object(object) => assert_eq!(object["text"], expected),
                Value::Array(array) => assert_eq!(array, vec![Value::String(expected.clone())]),
                value => assert_eq!(value, expected),
            }
        }
    }
}

#[test]
fn leading_backslashes_in_keys_follow_the_escape_policy() {
    for preserve in [false, true] {
        for decode in [false, true] {
            let policy = options(preserve, decode);
            for (input, expected) in [
                (r"{\key: 1}", r"\key"),
                (
                    r"{\\server: 1}",
                    if decode { r"\server" } else { r"\\server" },
                ),
                (
                    r"{\u0041lice: 1}",
                    if decode { "Alice" } else { r"\u0041lice" },
                ),
            ] {
                assert_eq!(
                    check_result(input, policy, &[1, 2]),
                    Some(json!({expected: 1}))
                );
            }
        }
    }
}

#[test]
fn quoted_unicode_and_control_characters_are_policy_independent() {
    for value in [
        "",
        "# // /* */",
        "C:\\new\\table.txt",
        "中文🦀",
        "e\u{0301}",
        "\u{0000}\u{0001}\u{0008}\u{000c}\n\r\t\u{001f}",
        "\"\\/{}[],+:;()",
        "\u{2028}\u{2029}\u{FEFF}",
    ] {
        let input = json!({"text": value, "items": [value]}).to_string();
        for preserve in [false, true] {
            for decode in [false, true] {
                let policy = options(preserve, decode);
                assert_eq!(jsonrepair_with_options(&input, policy).unwrap(), input);
                assert_eq!(
                    check_result(&input, policy, &[1, 2, 3]),
                    Some(json!({"text": value, "items": [value]}))
                );
            }
        }
    }
}

#[test]
fn all_bmp_unicode_escapes_obey_decoding_policy() {
    for code in 0..=u16::MAX {
        let encoded = format!("prefix\\u{code:04X}suffix");
        let input = format!("{{\"text\": {encoded}}}");
        for decode in [false, true] {
            let result = jsonrepair_with_options(&input, options(true, decode));
            if decode && char::from_u32(u32::from(code)).is_none() {
                let error = result.unwrap_err();
                assert_eq!(error.kind, JsonRepairErrorKind::InvalidUnicode);
                assert_eq!(error.position, input.find('\\').unwrap());
            } else {
                let expected = if decode {
                    format!("prefix{}suffix", char::from_u32(u32::from(code)).unwrap())
                } else {
                    encoded.clone()
                };
                let repaired = result.unwrap();
                let actual: Value = serde_json::from_str(&repaired).unwrap();
                assert_eq!(
                    actual,
                    json!({"text": expected}),
                    "code={code:04X}, decode={decode}"
                );
            }
        }
    }
    println!("checked all 65,536 BMP code units with decoding on and off");
}

#[test]
fn unicode_surrogate_pair_boundaries_decode_correctly() {
    for high in [0xD800u32, 0xD801, 0xDBFE, 0xDBFF] {
        for low in [0xDC00u32, 0xDC01, 0xDFFE, 0xDFFF] {
            let encoded = format!("prefix\\u{high:04X}\\u{low:04X}suffix");
            let scalar = 0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00);
            let expected = format!("prefix{}suffix", char::from_u32(scalar).unwrap());
            assert_eq!(
                check_result(
                    &format!("{{\"text\": {encoded}}}"),
                    options(true, true),
                    &[1, 2, 3]
                ),
                Some(json!({"text": expected}))
            );
        }
    }
}

#[test]
fn long_unquoted_tool_content_preserves_text_and_escapes() {
    let literal = "# Heading\\n中文🦀 // code /* example */ ".repeat(8_192) + "end";
    let input = format!("{{\"text\": {literal}}}");
    for decode in [false, true] {
        let expected = if decode {
            literal.replace(r"\n", "\n")
        } else {
            literal.clone()
        };
        assert_eq!(
            check_result(&input, options(true, decode), &[1, 7, 4096]),
            Some(json!({"text": expected}))
        );
    }
}

#[test]
fn transport_prefixes_keep_json_and_io_contracts() {
    for input in [
        r##"{"text": # Title\n中文\uD83D\uDE00\"}"##,
        r#"{"path": C:\new\table.txt, "other": /* text */}"#,
        r#"callback({"text": //example.com/path}, [foo\nbar])"#,
        r#"{"text": prefix\uD800\u0041}"#,
        "{\r\n\"text\": # Title\r\n## Sub\r\n}",
    ] {
        for end in input
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(input.len()))
        {
            for preserve in [false, true] {
                for decode in [false, true] {
                    check_result(&input[..end], options(preserve, decode), &[1, 3]);
                }
            }
        }
    }
}

#[test]
fn generated_malformed_tokens_keep_successful_output_valid() {
    let fragments = [
        "text",
        "中文🦀",
        "# heading",
        "//server",
        "/* block */",
        "\\",
        "\\\"",
        "\\n",
        "\\u0041",
        "\\uD800",
        "\\uDC00",
        "\\uD83D\\uDE00",
        "\\u12",
        "\\q",
        "\"",
        "'",
        "(",
        ")",
        "{",
        "}",
        "[",
        "]",
        ",",
        ":",
        "+",
        ";",
        "\r\n",
        "\u{000c}",
    ];
    let mut count = 0;
    for first in fragments {
        for second in fragments {
            for third in ["tail", "\\", "\"", "\r\n"] {
                let token = format!("{first}{second}{third}");
                for input in [format!("{{\"text\": {token}}}"), format!("[{token}]")] {
                    for preserve in [false, true] {
                        for decode in [false, true] {
                            check_result(&input, options(preserve, decode), &[]);
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    println!("checked {count} generated malformed-input/policy combinations");
}
