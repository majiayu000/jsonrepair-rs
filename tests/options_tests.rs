use jsonrepair_rs::{
    jsonrepair, jsonrepair_reader_to_writer_with_options, jsonrepair_to_writer_with_options,
    jsonrepair_with_options, JsonRepairErrorKind, RepairOptions,
};
use serde_json::{json, Value};

fn assert_policy(input: &str, options: RepairOptions, expected: Value) {
    let repaired = jsonrepair_with_options(input, options).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&repaired).unwrap(), expected);
    assert_eq!(
        jsonrepair_with_options(&repaired, options).unwrap(),
        repaired
    );
    assert_eq!(
        jsonrepair_with_options(&repaired, options.with_strict(true)).unwrap(),
        repaired
    );

    let mut written = Vec::new();
    jsonrepair_to_writer_with_options(input, &mut written, options).unwrap();
    assert_eq!(written, repaired.as_bytes());
    let mut streamed = Vec::new();
    jsonrepair_reader_to_writer_with_options(input.as_bytes(), &mut streamed, options).unwrap();
    assert_eq!(streamed, written);

    #[cfg(feature = "serde")]
    {
        assert_eq!(
            jsonrepair_rs::jsonrepair_value_with_options(input, options).unwrap(),
            expected
        );
        assert_eq!(
            jsonrepair_rs::jsonrepair_parse_with_options::<Value>(input, options).unwrap(),
            expected
        );
    }
}

#[test]
fn comment_and_escape_policies_are_independent() {
    for preserve in [false, true] {
        for decode in [false, true] {
            let options = RepairOptions::new()
                .with_preserve_comment_markers(preserve)
                .with_decode_unquoted_escapes(decode);
            for (input, literal) in [
                (r##"{"plan": # Title\nSub"}"##, r"# Title\nSub"),
                (r#"{"plan": // Title\nSub}"#, r"// Title\nSub"),
                (r#"{"plan": /* Title\nSub */}"#, r"/* Title\nSub */"),
            ] {
                let expected = if preserve {
                    json!({"plan": if decode { literal.replace(r"\n", "\n") } else { literal.to_owned() }})
                } else {
                    json!({"plan": null})
                };
                assert_policy(input, options, expected);
            }
            assert_policy(
                r#"{"plan": Title\nSub}"#,
                options,
                json!({"plan": if decode { "Title\nSub" } else { r"Title\nSub" }}),
            );
            assert_policy(
                r#"{"path": C:\new\table.txt}"#,
                options,
                json!({"path": if decode { "C:\new\table.txt" } else { r"C:\new\table.txt" }}),
            );
        }
    }
}

#[test]
fn comment_preservation_keeps_slashes_in_unquoted_content() {
    let options = RepairOptions::new().with_preserve_comment_markers(true);
    for content in [
        "//example.com/path",
        "/* text */",
        "text // example",
        "text /* example */",
        "# Heading",
        "https://example.com/path#section",
    ] {
        assert_policy(
            &format!("{{\"content\": {content}}}"),
            options,
            json!({"content": content}),
        );
    }
}

#[test]
fn unquoted_json_escapes_include_quotes_slashes_and_unicode() {
    let options = RepairOptions::new().with_decode_unquoted_escapes(true);
    assert_policy(
        r#"{"text": prefix\"quote\"\\\/\b\f\n\r\t\u0041\uD83D\uDE00}"#,
        options,
        json!({"text": "prefix\"quote\"\\/\u{0008}\u{000c}\n\r\tA😀"}),
    );
    assert_policy(r#"{key\u0041: value}"#, options, json!({"keyA": "value"}));
    assert_policy(
        r##"{"plan": # Title\n## Sub\nsome prose here\"}"##,
        options.with_preserve_comment_markers(true),
        json!({"plan": "# Title\n## Sub\nsome prose here\""}),
    );
}

#[test]
fn incomplete_and_non_json_unquoted_escapes_remain_literal() {
    let options = RepairOptions::new().with_decode_unquoted_escapes(true);
    for content in [r"value\q", r"value\uZZZZ", r"value\u12", r"value\"] {
        assert_policy(
            &format!("{{\"text\": {content}}}"),
            options,
            json!({"text": content}),
        );
    }
}

#[test]
fn decoding_unquoted_surrogates_preserves_unicode_error_contract() {
    let options = RepairOptions::new().with_decode_unquoted_escapes(true);
    for content in [r"value\uD800", r"value\uDC00", r"value\uD800\u0041"] {
        let input = format!("{{\"text\": {content}}}");
        let err = jsonrepair_with_options(&input, options).unwrap_err();
        assert_eq!(err.kind, JsonRepairErrorKind::InvalidUnicode);
        assert_eq!(err.position, input.find('\\').unwrap());
        assert_eq!(err.line, 1);
        assert_eq!(err.column, err.position + 1);
        let mut written = Vec::new();
        assert!(matches!(
            jsonrepair_to_writer_with_options(&input, &mut written, options),
            Err(jsonrepair_rs::JsonRepairWriteError::Repair(error))
                if error.kind == JsonRepairErrorKind::InvalidUnicode
        ));
        assert!(written.is_empty());
    }
}

#[test]
fn quoted_json_is_unchanged_under_all_policy_combinations() {
    let input = r##"{"text":"# // /* */\n\u0041","path":"C:\\new\\table.txt"}"##;
    for preserve in [false, true] {
        for decode in [false, true] {
            let options = RepairOptions::new()
                .with_preserve_comment_markers(preserve)
                .with_decode_unquoted_escapes(decode);
            assert_eq!(jsonrepair_with_options(input, options).unwrap(), input);
            assert_eq!(
                jsonrepair_with_options(input, options.with_strict(true)).unwrap(),
                input
            );
            let err = jsonrepair_with_options(r#"{"text": Title\nSub}"#, options.with_strict(true))
                .unwrap_err();
            assert_eq!(err.kind, JsonRepairErrorKind::StrictModeViolation);
        }
    }
}

#[test]
fn comment_preservation_does_not_enable_multiline_unquoted_strings() {
    for decode in [false, true] {
        let options = RepairOptions::new()
            .with_preserve_comment_markers(true)
            .with_decode_unquoted_escapes(decode);
        let input = "{\"plan\":\n# Title\n## Sub\n}";
        let err = jsonrepair_with_options(input, options).unwrap_err();
        assert_eq!(err.kind, JsonRepairErrorKind::ColonExpected);
        let mut streamed = Vec::new();
        assert!(matches!(
            jsonrepair_reader_to_writer_with_options(input.as_bytes(), &mut streamed, options),
            Err(jsonrepair_rs::JsonRepairStreamError::Repair(error))
                if error.kind == JsonRepairErrorKind::ColonExpected
        ));
        assert!(streamed.is_empty());
    }
}

#[test]
fn default_options_match_jsonrepair() {
    let input = "{name: 'Ada', active: True}";

    assert_eq!(
        jsonrepair_with_options(input, RepairOptions::default()).unwrap(),
        jsonrepair(input).unwrap()
    );
}

#[test]
fn strict_mode_passes_valid_json_unchanged() {
    let input = r#"{"name": "Ada", "active": true, "skills": ["rust"]}"#;

    assert_eq!(
        jsonrepair_with_options(input, RepairOptions::strict()).unwrap(),
        input
    );
}

#[test]
fn strict_mode_rejects_repairable_inputs() {
    for input in [
        "{name: 'Ada'}",
        "{\n  // comment\n  \"name\": \"Ada\"\n}",
        "```json\n{\"name\":\"Ada\"}\n```",
        "{\"active\": True}",
        "callback({\"name\":\"Ada\"});",
        "{\"value\": NaN}",
        "{\"items\": [1,2,]}",
        "{\"a\":1}\n{\"b\":2}",
    ] {
        let err = jsonrepair_with_options(input, RepairOptions::strict())
            .expect_err(&format!("expected strict error for {input:?}"));
        assert_eq!(err.kind, JsonRepairErrorKind::StrictModeViolation);
        assert!(err.line > 0);
        assert!(err.column > 0);
    }
}

#[test]
fn strict_mode_is_available_for_writer_helpers() {
    let mut output = Vec::new();
    let err =
        jsonrepair_to_writer_with_options("{name: 'Ada'}", &mut output, RepairOptions::strict())
            .unwrap_err();

    assert!(matches!(
        err,
        jsonrepair_rs::JsonRepairWriteError::Repair(repair_err)
            if repair_err.kind == JsonRepairErrorKind::StrictModeViolation
    ));
    assert!(output.is_empty());
}

#[test]
fn string_byte_limit_precedes_repair_and_writer_calls() {
    let input = "{text:'中'}";
    let options = RepairOptions::new().with_max_input_bytes(input.len());
    assert!(jsonrepair_with_options(input, options).is_ok());
    let mut writer = Vec::new();
    let err = jsonrepair_to_writer_with_options(
        input,
        &mut writer,
        RepairOptions::new().with_max_input_bytes(input.len() - 1),
    )
    .unwrap_err();
    assert!(
        matches!(err, jsonrepair_rs::JsonRepairWriteError::Repair(err)
        if err.kind == JsonRepairErrorKind::InputTooLarge)
    );
    assert!(writer.is_empty());
    assert!(jsonrepair_with_options("0", RepairOptions::new().with_max_input_bytes(0)).is_err());
    assert!(
        jsonrepair_with_options("0", RepairOptions::new().with_max_input_bytes(usize::MAX)).is_ok()
    );
}
