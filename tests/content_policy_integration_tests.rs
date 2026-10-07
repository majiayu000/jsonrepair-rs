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

fn assert_url_contexts(url: &str, expected: &str, policy: RepairOptions) {
    for (input, value) in [
        (
            format!("{{url:{url},next:2}}"),
            json!({"url":expected,"next":2}),
        ),
        (format!("[{url},2]"), json!([expected, 2])),
        (format!("cb({url})"), json!(expected)),
        (
            format!("{{x:cb(cb({url},2),a:b),next:3}}"),
            json!({"x":[[expected,2],"a:b"],"next":3}),
        ),
    ] {
        assert_eq!(repaired_value(&input, policy), value, "{input}");
    }
}

#[test]
fn url_parenthesis_depth_ends_with_raw_or_decoded_non_url_text() {
    assert_eq!(
        repaired_value("{url:https://x/a( text,next:2}", options(true, false)),
        json!({"url":"https://x/a( text","next":2})
    );
    for (preserve, decode) in [(true, false), (true, true), (false, true)] {
        for prefix in ["https://x/", "//x/"] {
            if !preserve && prefix.starts_with("//") {
                continue;
            }
            for (opening, decoded_opening) in [("a(", "a("), (r"a\u0028", "a(")] {
                if !decode && opening.contains('\\') {
                    continue;
                }
                for (separator, decoded_separator) in [
                    (" ", " "),
                    ("\t", "\t"),
                    (r"\u0020", " "),
                    (r"\t", "\t"),
                    (r"\n", "\n"),
                    (r"\u0001", "\u{0001}"),
                    (r"\u002C", ","),
                ] {
                    if !decode && separator.starts_with('\\') {
                        continue;
                    }
                    if !preserve && matches!(separator, " " | "\t") {
                        // Escape decoding does not broaden raw URL whitespace syntax.
                        continue;
                    }
                    assert_url_contexts(
                        &format!("{prefix}{opening}{separator}text"),
                        &format!("{prefix}{decoded_opening}{decoded_separator}text"),
                        options(preserve, decode),
                    );
                }
            }
        }
        // Ordinary wrapper text still owns its parentheses after URL mode ends.
        assert_eq!(
            repaired_value("{x:cb(text(a,b),2),next:3}", options(preserve, decode)),
            json!({"x":["text(a,b)",2],"next":3})
        );
        if preserve {
            assert_url_contexts(
                "https://x/a( text(b,c)",
                "https://x/a( text(b,c)",
                options(preserve, decode),
            );
        }
    }
}

#[test]
fn url_apostrophes_look_through_valid_decoded_url_characters() {
    assert_eq!(
        repaired_value(r"{url:https://x/O'\/path}", options(true, true)),
        json!({"url":"https://x/O'/path"})
    );
    for preserve in [true, false] {
        for prefix in ["https://x/", "//x/"] {
            if !preserve && prefix.starts_with("//") {
                continue;
            }
            for (suffix, decoded) in [
                (r"O'\/path", "O'/path"),
                (r"O'\u002Fpath", "O'/path"),
                (r"O'\u0052eilly", "O'Reilly"),
                (r"a('\u0062)", "a('b)"),
                (r"a'\u0028b)", "a'(b)"),
                (r"a('\u0029", "a(')"),
                (r"a'\u0029", "a')"),
                (r"a'\u0027b", "a''b"),
            ] {
                assert_url_contexts(
                    &format!("{prefix}{suffix}"),
                    &format!("{prefix}{decoded}"),
                    options(preserve, true),
                );
            }
            // A raw wrapper close still makes the preceding quote dangling.
            assert_eq!(
                repaired_value(&format!("cb({prefix}a')"), options(preserve, true)),
                json!(format!("{prefix}a"))
            );
        }
    }
}

#[test]
fn decoding_standard_urls_is_independent_of_comment_preservation() {
    assert_eq!(
        repaired_value(r"{url:https://example.com/a\/b}", options(false, true)),
        json!({"url":"https://example.com/a/b"})
    );
    for scheme in ["http", "https", "ftp", "mailto", "file", "data", "irc"] {
        for (suffix, decoded) in [
            (r"a\/b+c;v", "a/b+c;v"),
            (r"a\u002Fb+c;v", "a/b+c;v"),
            (r"a\u0028b)/c", "a(b)/c"),
            (r"a(b\u0029/c", "a(b)/c"),
            (r"a\u0028b\u0029/c", "a(b)/c"),
        ] {
            for preserve in [false, true] {
                assert_url_contexts(
                    &format!("{scheme}://example.com/{suffix}"),
                    &format!("{scheme}://example.com/{decoded}"),
                    options(preserve, true),
                );
            }
        }
    }
    for decode in [false, true] {
        assert_eq!(
            repaired_value("{url:// comment\n2,next:3}", options(false, decode)),
            json!({"url":2,"next":3})
        );
    }
    for suffix in [r"a\uD800", r"a\uDC00", r"a\uD800\u0041"] {
        let input = format!("{{url:https://x/{suffix}}}");
        for preserve in [false, true] {
            let error = jsonrepair_with_options(&input, options(preserve, true)).unwrap_err();
            assert_eq!(error.kind, JsonRepairErrorKind::InvalidUnicode);
            assert_eq!(error.position, input.find('\\').unwrap());
            assert_eq!(error.column, error.position + 1);
        }
    }
    for suffix in [r"a\u12", r"a\uZZZZ", r"a\"] {
        assert_url_contexts(
            &format!("https://x/{suffix}"),
            &format!("https://x/{suffix}"),
            options(false, true),
        );
    }
}

#[test]
fn decode_only_urls_retain_literal_escapes_and_default_raw_boundaries() {
    for suffix in [r"a\u12", r"a\uZZZZ", r"a\q/path", r"a\"] {
        for marker in ["#outside", "//outside", "/*outside*/"] {
            let input = format!("{{url:https://x/{suffix} {marker}\n,next:2}}");
            assert_eq!(
                repaired_value(&input, options(false, true)),
                json!({"url":format!("https://x/{suffix}"),"next":2})
            );
        }
    }
    for suffix in ["a( text", "a(\ttext", "a( text(b,c)"] {
        for input in [
            format!("{{url:https://x/{suffix},next:2}}"),
            format!("[https://x/{suffix},2]"),
            format!("cb(https://x/{suffix})"),
        ] {
            let outcome = |decode| {
                jsonrepair_with_options(&input, options(false, decode))
                    .map(|s| serde_json::from_str::<Value>(&s).unwrap())
                    .map_err(|e| (e.kind, e.position))
            };
            assert_eq!(outcome(true), outcome(false), "{input}");
        }
    }
}

#[test]
fn url_and_comment_boundaries_follow_each_policy_combination() {
    for preserve in [false, true] {
        for decode in [false, true] {
            let policy = options(preserve, decode);
            assert_eq!(
                repaired_value(r#"{text:word\nline\u0041}"#, policy),
                json!({"text":if decode { "word\nlineA" } else { r"word\nline\u0041" }})
            );
            // Markers inside scheme URLs were already URL content under defaults.
            for path in [
                "a#frag", "a//tag", "a/*tag*/", "a+b;c", "O'Reilly", "O'/path", "a('b)", "a((b))",
            ] {
                let url = format!("https://x/{path}");
                assert_eq!(
                    repaired_value(&format!("{{url:{url},next:2}}"), policy),
                    json!({"url":url,"next":2})
                );
                assert_eq!(
                    repaired_value(&format!("cb({url},2)"), policy),
                    json!([url, 2])
                );
            }
            // Leading protocol-relative URLs still obey the comment policy.
            assert_eq!(
                repaired_value(r"{url://x/a\/b}", policy),
                if preserve {
                    json!({"url":if decode { "//x/a/b" } else { r"//x/a\/b" }})
                } else {
                    json!({"url":null})
                }
            );
            for marker in ["/*outside*/", "//outside", "#outside"] {
                let input = format!("{{url:https://x/a {marker}\n,next:2}}");
                assert_eq!(
                    repaired_value(&input, policy),
                    json!({"url":if preserve { format!("https://x/a {marker}") } else { "https://x/a".to_owned() },"next":2}),
                    "{input}; preserve={preserve}, decode={decode}"
                );
            }
            let input = r"{url:https://x/a\/b}";
            if preserve || decode {
                assert_eq!(
                    repaired_value(input, policy),
                    json!({"url":if decode { "https://x/a/b" } else { r"https://x/a\/b" }})
                );
            } else {
                // Keep the existing default URL-continuation error contract.
                assert_eq!(
                    jsonrepair_with_options(input, policy).unwrap_err().kind,
                    JsonRepairErrorKind::ColonExpected
                );
            }
        }
    }
}

#[test]
fn url_depth_is_released_by_raw_and_decoded_non_url_characters() {
    for preserve in [false, true] {
        for decode in [false, true] {
            if !preserve && !decode {
                continue;
            }
            let policy = options(preserve, decode);
            for (suffix, expected) in [
                ("( text", "( text"),
                (r"\u0028 text", "( text"),
                (r"(\u0020text", "( text"),
                (r#"(\"text"#, "(\"text"),
            ] {
                if suffix.contains('\\') && !decode {
                    continue;
                }
                if !preserve && suffix.contains(' ') {
                    continue;
                }
                let url = format!("https://x/a{suffix}");
                let expected = format!("https://x/a{expected}");
                assert_eq!(
                    repaired_value(&format!("{{url:{url},next:2}}"), policy),
                    json!({"url":expected,"next":2})
                );
                assert_eq!(
                    repaired_value(&format!("cb({url},2)"), policy),
                    json!([expected, 2])
                );
            }
        }
    }
}

#[test]
fn decoded_url_successors_share_apostrophe_and_parenthesis_boundaries() {
    for preserve in [false, true] {
        let policy = options(preserve, true);
        for (path, expected) in [
            (r"O'\/path", "O'/path"),
            (r"O'\u002fpath", "O'/path"),
            (r"O'\u0041path", "O'Apath"),
            (r"O'\u0028b)", "O'(b)"),
            (r"O('b\u0029", "O('b)"),
            (r"O'\u0029", "O')"),
            (r"a\u0028(b)\u0029", "a((b))"),
        ] {
            let url = format!("https://x/{path}");
            let expected = format!("https://x/{expected}");
            assert_eq!(
                repaired_value(&format!("{{url:{url},next:2}}"), policy),
                json!({"url":expected,"next":2})
            );
            assert_eq!(
                repaired_value(&format!("[{url},2]"), policy),
                json!([expected, 2])
            );
            assert_eq!(
                repaired_value(&format!("{{x:cb({url},2),next:3}}"), policy),
                json!({"x":[expected,2],"next":3})
            );
        }
    }
}

#[test]
fn url_unicode_validation_keeps_kind_and_offset_under_each_policy() {
    for preserve in [false, true] {
        for decode in [false, true] {
            let policy = options(preserve, decode);
            for escape in [r"\uD800", r"\uDC00", r"\uD800\u0041"] {
                let input = format!("{{url:https://x/a{escape}}}");
                if decode {
                    let error = jsonrepair_with_options(&input, policy).unwrap_err();
                    assert_eq!(error.kind, JsonRepairErrorKind::InvalidUnicode);
                    assert_eq!(error.position, input.find('\\').unwrap());
                    assert_eq!(error.line, 1);
                    assert_eq!(error.column, error.position + 1);
                } else if preserve {
                    assert_eq!(
                        repaired_value(&input, policy),
                        json!({"url":format!("https://x/a{escape}")})
                    );
                } else {
                    assert_eq!(
                        jsonrepair_with_options(&input, policy).unwrap_err().kind,
                        JsonRepairErrorKind::ColonExpected
                    );
                }
            }
        }
    }
}
