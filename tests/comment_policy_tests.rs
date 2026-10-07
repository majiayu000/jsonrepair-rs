use std::io::{self, Cursor, Read};

use jsonrepair_rs::{
    jsonrepair, jsonrepair_reader_to_writer_with_options, jsonrepair_to_writer_with_options,
    jsonrepair_with_options, JsonRepairErrorKind, RepairOptions,
};

fn content_options() -> RepairOptions {
    RepairOptions::new().with_preserve_comment_markers(true)
}

#[test]
fn comment_preservation_is_independent_and_reversible() {
    let options = content_options().with_preserve_comment_markers(false);
    assert_eq!(options, RepairOptions::default());
    let input = r##"{"plan": # Markdown heading"}"##;
    assert_eq!(
        jsonrepair_with_options(input, options).unwrap(),
        jsonrepair(input).unwrap()
    );
    assert_eq!(
        content_options().with_strict(true).with_strict(false),
        content_options()
    );
}

fn assert_content(input: &str, expected: &str) {
    let repaired = jsonrepair_with_options(input, content_options()).unwrap();
    assert_eq!(repaired, expected);
    assert_eq!(jsonrepair(&repaired).unwrap(), repaired);
    assert_eq!(
        jsonrepair_with_options(&repaired, RepairOptions::strict()).unwrap(),
        repaired
    );
}

#[test]
fn preserves_hash_heading() {
    assert_content(
        r##"{"plan": # Markdown heading"}"##,
        r##"{"plan": "# Markdown heading"}"##,
    );
}

#[test]
fn preserves_protocol_relative_url() {
    assert_content(
        r#"{"url": //example.com/path}"#,
        r#"{"url": "//example.com/path"}"#,
    );
}

#[test]
fn preserves_protocol_relative_url_delimiters() {
    for url in [
        "//example.com/a+b",
        "//example.com/a;b",
        "//example.com/a+b;c?x=1#section",
        "//example.com/path(with+parens);v=1",
    ] {
        assert_content(
            &format!("{{\"url\": {url}}}"),
            &format!("{{\"url\": \"{url}\"}}"),
        );
        assert_content(&format!("[{url}, true]"), &format!("[\"{url}\", true]"));
    }
}

#[test]
fn protocol_relative_urls_keep_wrapper_boundaries() {
    assert_content(
        r#"{x:cb(1,//example.com/a+b;c,2),y:3}"#,
        r#"{"x":[1,"//example.com/a+b;c",2],"y":3}"#,
    );
    assert_content(
        r#"{x:cb(1,{url://example.com/a+b;c},a:b),y:2}"#,
        r#"{"x":[1,{"url":"//example.com/a+b;c"},"a:b"],"y":2}"#,
    );
    assert_content(
        r#"{x:cb(1,//example.com/path(with+parens);v=1),y:2}"#,
        r#"{"x":[1,"//example.com/path(with+parens);v=1"],"y":2}"#,
    );
}

#[test]
fn protocol_relative_url_parentheses_keep_json_boundaries() {
    for (input, expected) in [
        (
            r#"{"url": //example.com/a(}"#,
            r#"{"url": "//example.com/a("}"#,
        ),
        (
            r#"{"url": //example.com/a(b+c;d,"next":true}"#,
            r#"{"url": "//example.com/a(b+c;d","next":true}"#,
        ),
        (r#"[//example.com/a(]"#, r#"["//example.com/a("]"#),
        (
            r#"[//example.com/a(b+c;d, true]"#,
            r#"["//example.com/a(b+c;d", true]"#,
        ),
        (
            r#"{"url": //example.com/a("}"#,
            r#"{"url": "//example.com/a("}"#,
        ),
        (
            r#"{"url": //example.com/a('}"#,
            r#"{"url": "//example.com/a("}"#,
        ),
    ] {
        assert_content(input, expected);
        assert_eq!(
            jsonrepair_with_options(input, content_options().with_strict(true)),
            jsonrepair_with_options(input, RepairOptions::strict()),
            "strict precedence: {input}"
        );
    }
}

#[test]
fn protocol_relative_url_parentheses_keep_nested_wrapper_ownership() {
    assert_content(
        r#"{x:cb(1,{url://example.com/a(},a:b),y:2}"#,
        r#"{"x":[1,{"url":"//example.com/a("},"a:b"],"y":2}"#,
    );
    assert_content(
        r#"{x:cb(1,[//example.com/a(],a:b),y:2}"#,
        r#"{"x":[1,["//example.com/a("],"a:b"],"y":2}"#,
    );
}

#[test]
fn wrapper_url_boundaries_follow_numeric_and_symbol_keys() {
    for key in ["123", "-1", "@tag"] {
        for separator in [':', '='] {
            assert_content(
                &format!("{{x:cb(1,{{{key}{separator}//example.com/a(}},a:b),y:2}}"),
                &format!("{{\"x\":[1,{{\"{key}\":\"//example.com/a(\"}},\"a:b\"],\"y\":2}}"),
            );
        }
    }
}

#[test]
fn non_url_comment_text_keeps_parenthesized_content() {
    for text in [
        "// literal(foo } text)",
        "//example.com/a (text ] rest)",
        "keep //example.com/a(} text)",
        "// literal(foo + bar; } text)",
    ] {
        assert_content(
            &format!("{{\"note\": {text}}}"),
            &format!("{{\"note\": \"{text}\"}}"),
        );
    }
}

#[test]
fn preserves_block_marker_text() {
    assert_content(
        r#"{"pattern": /* literal text */}"#,
        r#"{"pattern": "/* literal text */"}"#,
    );
}

#[test]
fn preserves_markers_in_arrays_and_nested_values() {
    assert_content(
        r##"{"items":[# first,//example.com/path,/*literal*/],"nested":{"note":# 中文 🦀}}"##,
        r##"{"items":["# first","//example.com/path","/*literal*/"],"nested":{"note":"# 中文 🦀"}}"##,
    );
}

#[test]
fn preserves_comment_markers_after_other_text() {
    assert_content(
        r##"{"note": keep // and /* markers */ # literal}"##,
        r##"{"note": "keep // and /* markers */ # literal"}"##,
    );
    assert_content(
        r##"{"note": foo # heading/path}"##,
        r##"{"note": "foo # heading/path"}"##,
    );
}

#[test]
fn keeps_unquoted_backslashes_literal() {
    assert_content(
        r#"{"path": C:\new\table.txt}"#,
        r#"{"path": "C:\\new\\table.txt"}"#,
    );
    assert_content(
        r##"{"plan": # Title\n## Sub\t\u0041lice}"##,
        r##"{"plan": "# Title\\n## Sub\\t\\u0041lice"}"##,
    );
}

#[test]
fn keeps_default_comment_stripping() {
    for input in [
        r##"{"content": # Release notes"}"##,
        r##"{"content": // Release notes"}"##,
        r#"{"content": //example.com/a+b;c}"#,
        r##"{"content": /* Release notes */}"##,
    ] {
        assert_eq!(jsonrepair(input).unwrap(), r#"{"content": null}"#);
        assert_eq!(
            jsonrepair_with_options(input, RepairOptions::default()).unwrap(),
            jsonrepair(input).unwrap()
        );
    }
    let input = "{\n# config\nname:'demo', // line\nactive:True /* block */\n}";
    assert_eq!(
        jsonrepair(input).unwrap(),
        "{\n\n\"name\":\"demo\", \n\"active\":true \n}"
    );
}

#[test]
fn keeps_valid_quoted_json_identical() {
    let input = r##"{"plan":"# Title\n// example\n/* text */","path":"C:\\new\\table.txt"}"##;
    assert_content(input, input);
}

#[test]
fn keeps_recognized_url_delimiters() {
    for scheme in ["http", "https", "ftp", "file"] {
        let input = format!("{{\"url\":{scheme}://example.org/a+b;c?x=1#section}}");
        let expected = format!("{{\"url\":\"{scheme}://example.org/a+b;c?x=1#section\"}}");
        assert_content(&input, &expected);
    }
}

#[test]
fn keeps_regex_repair_for_non_comment_slashes() {
    for input in [
        r#"{"regex": /a,b/g}"#,
        r#"{"regex": /[a-z]+/g}"#,
        r#"{"regex": /a\/b/g}"#,
        r#"[/a,b/g, /[a-z]+/g]"#,
        r#"/foo"#,
        r#"callback(1,/foo)"#,
        r#"callback(1,/[)]/i)"#,
    ] {
        let expected = jsonrepair(input).unwrap();
        assert_content(input, &expected);
    }
}

#[test]
fn keeps_slash_boundaries_before_any_comment_marker() {
    for input in [
        r#"{"note": foo/bar}"#,
        r#"{"note": foo /bar/}"#,
        r#"[foo/bar, baz]"#,
        r#"cb(foo/bar, baz)"#,
    ] {
        assert_eq!(
            jsonrepair_with_options(input, content_options()),
            jsonrepair(input),
            "{input}"
        );
    }
}

#[test]
fn wrapper_lookahead_keeps_regex_arguments() {
    for regex in [r"/a,b/g", r"/[a-z]+/g", r"/[{}()]/i", r"/\}\)\//g"] {
        let input = format!("{{x:callback(1,{regex},a:b)}}");
        let expected = jsonrepair(&input).unwrap();
        assert_content(&input, &expected);
    }
}

#[test]
fn nested_unquoted_keys_keep_regex_boundaries_in_wrapper_lookahead() {
    for regex in [r"/}/", r"/[}]/g", r"/[\/]/g", r"/a,b/g"] {
        let input = format!("{{x:cb(1,{{r:{regex}}},a:b),y:2}}");
        let expected = jsonrepair(&input).unwrap();
        assert_content(&input, &expected);
    }
}

#[test]
fn preserves_comment_text_alongside_regex_values() {
    assert_content(
        r##"{"regex": /a,b/g, "url": //example.com/path, "note": # heading}"##,
        r##"{"regex": "/a,b/g", "url": "//example.com/path", "note": "# heading"}"##,
    );
    assert_content(
        r#"{"x": cb(1, /[a-z]+/g, /* literal text */), "y": 2}"#,
        r#"{"x": [1, "/[a-z]+/g", "/* literal text */"], "y": 2}"#,
    );
}

#[test]
fn wrapper_lookahead_keeps_marker_arguments_inside_the_wrapper() {
    for marker in ["#tag", "//tag", "/*tag"] {
        let input = format!("{{\"x\": cb(1, {marker}: hello), \"y\": 2}}");
        let expected = format!("{{\"x\": [1, \"{marker}: hello\"], \"y\": 2}}");
        assert_content(&input, &expected);
    }
}

#[test]
fn strict_mode_takes_precedence() {
    let options = content_options().with_strict(true);
    let valid = r##"{"value":"# literal"}"##;
    assert_eq!(jsonrepair_with_options(valid, options).unwrap(), valid);
    for input in [
        r##"{"value": # literal}"##,
        r#"{"value": //example.com/path}"#,
        r#"{"value": /* literal */}"#,
    ] {
        assert_eq!(
            jsonrepair_with_options(input, options).unwrap_err().kind,
            JsonRepairErrorKind::StrictModeViolation
        );
    }
}

#[test]
fn writer_and_reader_forward_comment_policy() {
    let input = r##"{"plan":# 中文 🦀,"path":C:\new\table.txt}"##;
    let expected = r##"{"plan":"# 中文 🦀","path":"C:\\new\\table.txt"}"##;
    let mut output = Vec::new();
    jsonrepair_to_writer_with_options(input, &mut output, content_options()).unwrap();
    assert_eq!(output, expected.as_bytes());

    struct Chunked<'a>(Cursor<&'a [u8]>, usize);
    impl Read for Chunked<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let len = buffer.len().min(self.1);
            self.0.read(&mut buffer[..len])
        }
    }
    for chunk_size in 1..=5 {
        let reader = Chunked(Cursor::new(input.as_bytes()), chunk_size);
        let mut streamed = Vec::new();
        jsonrepair_reader_to_writer_with_options(reader, &mut streamed, content_options()).unwrap();
        assert_eq!(streamed, expected.as_bytes(), "chunk size {chunk_size}");
    }
}

#[test]
fn does_not_join_real_multiline_values() {
    let input = "{\"plan\":\n# Title\n## Sub\n}";
    let mut output = b"existing".to_vec();
    assert!(jsonrepair_to_writer_with_options(input, &mut output, content_options()).is_err());
    assert_eq!(output, b"existing");
}

#[cfg(feature = "serde")]
#[test]
fn serde_helpers_forward_comment_policy() {
    let input = r##"{"plan":# 中文 🦀,"path":C:\new\table.txt}"##;
    let expected = serde_json::json!({"plan": "# 中文 🦀", "path": r"C:\new\table.txt"});
    assert_eq!(
        jsonrepair_rs::jsonrepair_value_with_options(input, content_options()).unwrap(),
        expected
    );
    assert_eq!(
        jsonrepair_rs::jsonrepair_parse_with_options::<serde_json::Value>(input, content_options())
            .unwrap(),
        expected
    );
}
