use jsonrepair_rs::{jsonrepair_with_options, JsonRepairErrorKind, RepairOptions};
use serde_json::{json, Value};

fn options(preserve: bool, decode: bool) -> RepairOptions {
    RepairOptions::new()
        .with_preserve_comment_markers(preserve)
        .with_decode_unquoted_escapes(decode)
}

fn repaired_value(input: &str, policy: RepairOptions) -> Value {
    let repaired = jsonrepair_with_options(input, policy).unwrap();
    let value = serde_json::from_str(&repaired).unwrap();
    assert_eq!(
        jsonrepair_with_options(&repaired, policy.with_strict(true)).unwrap(),
        repaired
    );
    value
}

#[test]
fn wrapper_url_and_regex_repairs_keep_escape_decoding_independent() {
    let input = r##"{x:cb(1,{url://example.com/a(,regex:/a,b/g,note:# Title\nLine},a:b),y:2}"##;
    for decode in [false, true] {
        let note = if decode {
            "# Title\nLine"
        } else {
            r"# Title\nLine"
        };
        assert_eq!(
            repaired_value(input, options(true, decode)),
            json!({"x":[1,{"url":"//example.com/a(","regex":"/a,b/g","note":note},"a:b"],"y":2})
        );
    }
}

#[test]
fn decoded_url_escapes_remain_content_inside_nested_wrappers() {
    let input = r#"{x:cb(1,{url://example.com/a\/b+c;v,q://example.com/a\"b},a:b),y:2}"#;
    assert_eq!(
        repaired_value(input, options(true, true)),
        json!({"x":[1,{"url":"//example.com/a/b+c;v","q":"//example.com/a\"b"},"a:b"],"y":2})
    );
    assert_eq!(
        repaired_value(r#"{url:https://example.com/a\/b}"#, options(true, true)),
        json!({"url":"https://example.com/a/b"})
    );
    assert_eq!(
        repaired_value(r#"{url:https://example.com/a\/b+c;v}"#, options(true, true)),
        json!({"url":"https://example.com/a/b+c;v"})
    );
}

#[test]
fn strict_mode_takes_precedence_over_both_content_options() {
    for input in [
        r#"{"url": //example.com/a(}"#,
        r#"{"text": value\uD800}"#,
        r#"{x:cb(1,{123://example.com/a(},a:b),y:2}"#,
    ] {
        let expected = jsonrepair_with_options(input, RepairOptions::strict());
        assert!(expected.is_err());
        for preserve in [false, true] {
            for decode in [false, true] {
                assert_eq!(
                    jsonrepair_with_options(input, options(preserve, decode).with_strict(true)),
                    expected,
                    "{input}; preserve={preserve}, decode={decode}"
                );
            }
        }
    }
}

#[test]
fn decoded_non_url_text_ends_the_url_delimiter_exception() {
    for input in [
        r#"{url://example.com/a\u0020b+c}"#,
        r#"{url://example.com/a b+c}"#,
        r#"{url:https://example.com/a\u0020b+c}"#,
        r#"{url:https://example.com/a b+c}"#,
    ] {
        let error = jsonrepair_with_options(input, options(true, true)).unwrap_err();
        assert_eq!(error.kind, JsonRepairErrorKind::ObjectKeyExpected);
    }
}

#[test]
fn escape_decoding_does_not_reinterpret_a_url_prefix_as_syntax() {
    for (escaped, plain) in [
        (r"https:\/\/example.com", "https://example.com"),
        (r"\/\/example.com", "//example.com"),
    ] {
        for decode in [false, true] {
            let input = format!("{{url:{escaped}/abc}}");
            let prefix = if decode { plain } else { escaped };
            assert_eq!(
                repaired_value(&input, options(true, decode)),
                json!({"url":format!("{prefix}/abc")})
            );
            let error =
                jsonrepair_with_options(&format!("{{url:{escaped}/a+b;c}}"), options(true, decode))
                    .unwrap_err();
            assert_eq!(error.kind, JsonRepairErrorKind::ObjectKeyExpected);
            assert_eq!(
                repaired_value(&format!("{{url:{plain}/a+b;c}}"), options(true, decode)),
                json!({"url":format!("{plain}/a+b;c")})
            );
        }
        assert_eq!(
            repaired_value(
                &format!(r#"{{"url":"{escaped}/a+b;c"}}"#),
                options(true, true)
            ),
            json!({"url":format!("{plain}/a+b;c")})
        );
    }
}

#[test]
fn recognized_urls_preserve_apostrophes_without_swallowing_dangling_quotes() {
    for decode in [false, true] {
        for url in [
            "https://example.com/O'Reilly",
            "//example.com/O'Reilly",
            "https://example.com/a('b)",
        ] {
            assert_eq!(
                repaired_value(&format!("{{url:{url}}}"), options(true, decode)),
                json!({"url":url})
            );
            assert_eq!(
                repaired_value(&format!("[{url},2]"), options(true, decode)),
                json!([url, 2])
            );
            assert_eq!(
                repaired_value(&format!("{{x:cb({url},2)}}"), options(true, decode)),
                json!({"x":[url,2]})
            );
        }
        for quote in ["'", "\""] {
            assert_eq!(
                repaired_value(
                    &format!("{{url:https://example.com/path{quote},next:2}}"),
                    options(true, decode)
                ),
                json!({"url":"https://example.com/path","next":2})
            );
            assert_eq!(
                repaired_value(
                    &format!("[https://example.com/path{quote},2]"),
                    options(true, decode)
                ),
                json!(["https://example.com/path", 2])
            );
            assert_eq!(
                repaired_value(
                    &format!("{{x:cb(https://example.com/path{quote})}}"),
                    options(true, decode)
                ),
                json!({"x":"https://example.com/path"})
            );
            assert_eq!(
                repaired_value(
                    &format!("https://example.com/path{quote}"),
                    options(true, decode)
                ),
                json!("https://example.com/path")
            );
        }
    }
}
