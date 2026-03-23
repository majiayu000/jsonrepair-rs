use jsonrepair_rs::jsonrepair;

// ── Helpers ──────────────────────────────────────────────────

fn ok(input: &str, expected: &str) {
    let result = jsonrepair(input).unwrap_or_else(|e| panic!("repair failed for {input:?}: {e}"));
    assert_eq!(result, expected, "input: {input:?}");
}

fn err(input: &str) {
    assert!(
        jsonrepair(input).is_err(),
        "expected error for {input:?}, got: {:?}",
        jsonrepair(input)
    );
}

// ── 1. Valid JSON (pass-through, whitespace preserved) ───────

#[test]
fn valid_object() {
    ok(r#"{"a":2}"#, r#"{"a":2}"#);
}

#[test]
fn valid_object_with_spaces() {
    ok(r#"{"a": 2}"#, r#"{"a": 2}"#);
}

#[test]
fn valid_array() {
    ok("[1,2,3]", "[1,2,3]");
}

#[test]
fn valid_array_with_spaces() {
    ok("[ 1 , 2 , 3 ]", "[ 1 , 2 , 3 ]");
}

#[test]
fn valid_string() {
    ok(r#""hello""#, r#""hello""#);
}

#[test]
fn valid_number() {
    ok("42", "42");
}

#[test]
fn valid_negative_float() {
    ok("-3.14", "-3.14");
}

#[test]
fn valid_keywords() {
    ok("true", "true");
    ok("false", "false");
    ok("null", "null");
}

#[test]
fn valid_nested() {
    ok(
        r#"{"a":[1,{"b":true}],"c":null}"#,
        r#"{"a":[1,{"b":true}],"c":null}"#,
    );
}

#[test]
fn valid_empty_object_with_spaces() {
    ok("{  }", "{  }");
}

// ── 2. Quote repairs (whitespace preserved) ──────────────────

#[test]
fn single_quotes_to_double() {
    ok("{'name': 'John'}", r#"{"name": "John"}"#);
}

#[test]
fn single_quotes_value() {
    ok("'hello'", r#""hello""#);
}

#[test]
fn curly_double_quotes() {
    ok("\u{201C}hello\u{201D}", r#""hello""#);
}

#[test]
fn curly_single_quotes() {
    ok("\u{2018}hello\u{2019}", r#""hello""#);
}

#[test]
fn backtick_quotes() {
    ok("`hello`", r#""hello""#);
}

#[test]
fn unquoted_keys() {
    ok(r#"{name: "John"}"#, r#"{"name": "John"}"#);
}

// ── 3. String repairs ────────────────────────────────────────

#[test]
fn escape_unescaped_newline_in_string() {
    ok("\"hello\nworld\"", r#""hello\nworld""#);
}

#[test]
fn escape_unescaped_tab_in_string() {
    ok("\"hello\tworld\"", r#""hello\tworld""#);
}

#[test]
fn invalid_escape_sequence() {
    ok(r#""hello\a""#, r#""helloa""#);
}

#[test]
fn string_concatenation() {
    ok(r#""hello" + " " + "world""#, r#""hello world""#);
}

#[test]
fn incomplete_unicode_escape() {
    ok(r#""\u00""#, r#""\u0000""#);
}

// ── 4. Missing commas (whitespace preserved) ─────────────────

#[test]
fn missing_comma_between_object_properties() {
    ok(r#"{"a": 1 "b": 2}"#, r#"{"a": 1, "b": 2}"#);
}

#[test]
fn missing_comma_between_array_elements() {
    ok("[1 2 3]", "[1, 2, 3]");
}

// ── 5. Trailing commas (comma removed, whitespace preserved) ─

#[test]
fn trailing_comma_in_object() {
    ok(r#"{"a": 1, "b": 2,}"#, r#"{"a": 1, "b": 2}"#);
}

#[test]
fn trailing_comma_in_array() {
    ok("[1, 2, 3,]", "[1, 2, 3]");
}

// ── 6. Leading commas ────────────────────────────────────────

#[test]
fn leading_comma_in_object() {
    ok(r#"{,"a": 1}"#, r#"{"a": 1}"#);
}

#[test]
fn leading_comma_in_array() {
    ok("[,1,2]", "[1,2]");
}

// ── 7. Missing colon ────────────────────────────────────────

#[test]
fn missing_colon_no_space() {
    ok(r#"{"a"1}"#, r#"{"a":1}"#);
}

#[test]
fn missing_colon_with_space() {
    // Whitespace after key is consumed before colon insertion point
    ok(r#"{"a" 1}"#, r#"{"a ":1}"#);
}

#[test]
fn equals_as_colon() {
    // Whitespace before = is consumed, = replaced with :
    ok(r#"{"a" = 1}"#, r#"{"a ": 1}"#);
}

// ── 8. Missing value ────────────────────────────────────────

#[test]
fn missing_value_in_object() {
    ok(r#"{"a":}"#, r#"{"a":null}"#);
}

// ── 9. Comments (removed, surrounding whitespace preserved) ──

#[test]
fn line_comment() {
    ok("{\n// comment\n\"a\": 1\n}", "{\n\n\"a\": 1\n}");
}

#[test]
fn block_comment() {
    ok("{/* comment */\"a\": 1}", "{\"a\": 1}");
}

#[test]
fn hash_comment() {
    ok("{\n# comment\n\"a\": 1\n}", "{\n\n\"a\": 1\n}");
}

// ── 10. Python keywords ──────────────────────────────────────

#[test]
fn python_true() {
    ok("True", "true");
}

#[test]
fn python_false() {
    ok("False", "false");
}

#[test]
fn python_none() {
    ok("None", "null");
}

#[test]
fn python_keywords_in_object() {
    ok(
        r#"{"flag": True, "value": None}"#,
        r#"{"flag": true, "value": null}"#,
    );
}

// ── 11. JavaScript keywords ──────────────────────────────────

#[test]
fn js_undefined() {
    ok("undefined", "null");
}

#[test]
fn js_nan() {
    ok("NaN", "null");
}

#[test]
fn js_infinity() {
    ok("Infinity", "null");
}

// ── 12. Truncated JSON ───────────────────────────────────────

#[test]
fn truncated_object() {
    // Truncated: auto-close strips last comma, whitespace preserved
    ok(r#"{"a": 1, "b": 2"#, r#"{"a": 1 "b": 2}"#);
}

#[test]
fn truncated_array() {
    // Truncated: auto-close strips last comma, whitespace preserved
    ok("[1, 2, 3", "[1, 2 3]");
}

#[test]
fn truncated_string() {
    ok(r#""hello"#, r#""hello""#);
}

#[test]
fn truncated_nested() {
    ok(r#"{"a": [1, 2, {"b": 3"#, r#"{"a": [1 2 {"b": 3}]}"#);
}

// ── 13. Markdown code fences ─────────────────────────────────

#[test]
fn markdown_json_fence() {
    ok("```json\n{\"a\": 1}\n```", "{\"a\": 1}");
}

#[test]
fn markdown_plain_fence() {
    ok("```\n{\"a\": 1}\n```", "{\"a\": 1}");
}

// ── 14. Numbers ──────────────────────────────────────────────

#[test]
fn leading_zeros_as_string() {
    ok("0789", r#""0789""#);
}

#[test]
fn trailing_dot() {
    ok("2.", "2.0");
}

#[test]
fn truncated_exponent() {
    ok("2e", "2e0");
}

#[test]
fn number_with_exponent() {
    ok("1e5", "1e5");
}

#[test]
fn negative_number() {
    ok("-42", "-42");
}

// ── 15. MongoDB constructors ─────────────────────────────────

#[test]
fn mongodb_object_id() {
    ok(r#"{"_id": ObjectId("123abc")}"#, r#"{"_id": "123abc"}"#);
}

#[test]
fn mongodb_number_long() {
    ok(r#"{"count": NumberLong("42")}"#, r#"{"count": "42"}"#);
}

// ── 16. JSONP ────────────────────────────────────────────────

#[test]
fn jsonp_callback() {
    ok(r#"callback({"a": 1})"#, r#"{"a": 1}"#);
}

#[test]
fn trailing_semicolon() {
    ok(r#"{"a": 1};"#, r#"{"a": 1}"#);
}

// ── 17. Ellipsis ─────────────────────────────────────────────

#[test]
fn ellipsis_in_array() {
    ok("[1,2,3,...]", "[1,2,3]");
}

#[test]
fn ellipsis_in_array_with_spaces() {
    // Comma before ellipsis stripped, ellipsis consumed, trailing whitespace preserved
    ok("[1, 2, 3, ... ]", "[1, 2, 3  ]");
}

#[test]
fn ellipsis_in_object() {
    // Comma before ellipsis stripped, trailing space before } preserved
    ok(r#"{"a": 1, ...}"#, r#"{"a": 1 }"#);
}

// ── 18. BOM handling ─────────────────────────────────────────

#[test]
fn bom_prefix() {
    ok("\u{FEFF}{\"a\": 1}", "{\"a\": 1}");
}

// ── 19. Special whitespace ───────────────────────────────────

#[test]
fn non_breaking_space() {
    // NBSP is treated as whitespace and copied to output as-is
    ok("{\u{00A0}\"a\": 1}", "{\u{00A0}\"a\": 1}");
}

// ── 20. NDJSON ───────────────────────────────────────────────

#[test]
fn ndjson_two_objects() {
    // First value captures trailing \n, values joined with ,\n
    ok("{\"a\":1}\n{\"b\":2}", "[\n{\"a\":1}\n,\n{\"b\":2}\n]");
}

// ── 21. Error cases ──────────────────────────────────────────

#[test]
fn empty_input() {
    err("");
}

#[test]
fn whitespace_only() {
    err("   ");
}

// ── 22. Complex / mixed repairs ──────────────────────────────

#[test]
fn mixed_repairs() {
    ok(
        "{'name': 'John', age: 30, 'active': True,}",
        r#"{"name": "John", "age": 30, "active": true}"#,
    );
}

#[test]
fn deeply_nested_with_issues() {
    // Single-quoted 'hello' followed by space: space is consumed before comma insertion
    ok(
        "{a: [1, {b: 'hello' c: True},]}",
        r#"{"a": [1, {"b": "hello ","c": true}]}"#,
    );
}

#[test]
fn object_with_comments_and_trailing_commas() {
    ok(
        "{\n  // name\n  \"name\": \"John\",\n  /* age */\n  \"age\": 30,\n}",
        "{\n  \n  \"name\": \"John\",\n  \n  \"age\": 30\n}",
    );
}
