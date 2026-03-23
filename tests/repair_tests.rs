use jsonrepair_rs::jsonrepair;

// ── Helper ──────────────────────────────────────────────────

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

// ── 1. Valid JSON (pass-through) ────────────────────────────

#[test]
fn valid_object() {
    ok(r#"{"a":1,"b":2}"#, r#"{"a":1,"b":2}"#);
}

#[test]
fn valid_array() {
    ok("[1,2,3]", "[1,2,3]");
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
fn valid_negative_number() {
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

// ── 2. Quote repairs ────────────────────────────────────────

#[test]
fn single_quotes_to_double() {
    ok("{'name': 'John'}", r#"{"name":"John"}"#);
}

#[test]
fn single_quotes_value() {
    ok("'hello'", r#""hello""#);
}

#[test]
fn curly_double_quotes() {
    ok(
        "\u{201C}hello\u{201D}",
        r#""hello""#,
    );
}

#[test]
fn curly_single_quotes() {
    ok(
        "\u{2018}hello\u{2019}",
        r#""hello""#,
    );
}

#[test]
fn backtick_quotes() {
    ok("`hello`", r#""hello""#);
}

#[test]
fn unquoted_keys() {
    ok("{name: \"John\"}", r#"{"name":"John"}"#);
}

// ── 3. String repairs ───────────────────────────────────────

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

// ── 4. Missing commas ───────────────────────────────────────

#[test]
fn missing_comma_between_object_properties() {
    ok(
        r#"{"a": 1 "b": 2}"#,
        r#"{"a":1,"b":2}"#,
    );
}

#[test]
fn missing_comma_between_array_elements() {
    ok("[1 2 3]", "[1,2,3]");
}

// ── 5. Trailing commas ──────────────────────────────────────

#[test]
fn trailing_comma_in_object() {
    ok(r#"{"a": 1, "b": 2,}"#, r#"{"a":1,"b":2}"#);
}

#[test]
fn trailing_comma_in_array() {
    ok("[1, 2, 3,]", "[1,2,3]");
}

// ── 6. Leading commas ───────────────────────────────────────

#[test]
fn leading_comma_in_object() {
    ok(r#"{,"a": 1}"#, r#"{"a":1}"#);
}

#[test]
fn leading_comma_in_array() {
    ok("[,1,2]", "[1,2]");
}

// ── 7. Missing colon ───────────────────────────────────────

#[test]
fn missing_colon_in_object() {
    ok(r#"{"a" 1}"#, r#"{"a":1}"#);
}

#[test]
fn equals_as_colon() {
    ok(r#"{"a" = 1}"#, r#"{"a":1}"#);
}

// ── 8. Missing value ───────────────────────────────────────

#[test]
fn missing_value_in_object() {
    ok(r#"{"a":}"#, r#"{"a":null}"#);
}

// ── 9. Comments ─────────────────────────────────────────────

#[test]
fn line_comment() {
    ok(
        "{\n// comment\n\"a\": 1\n}",
        r#"{"a":1}"#,
    );
}

#[test]
fn block_comment() {
    ok(
        "{/* comment */\"a\": 1}",
        r#"{"a":1}"#,
    );
}

#[test]
fn hash_comment() {
    ok(
        "{\n# comment\n\"a\": 1\n}",
        r#"{"a":1}"#,
    );
}

// ── 10. Python keywords ─────────────────────────────────────

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
        r#"{"flag":true,"value":null}"#,
    );
}

// ── 11. JavaScript keywords ─────────────────────────────────

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

// ── 12. Truncated JSON ──────────────────────────────────────

#[test]
fn truncated_object() {
    ok(r#"{"a": 1, "b": 2"#, r#"{"a":1,"b":2}"#);
}

#[test]
fn truncated_array() {
    ok("[1, 2, 3", "[1,2,3]");
}

#[test]
fn truncated_string() {
    ok(r#""hello"#, r#""hello""#);
}

#[test]
fn truncated_nested() {
    ok(
        r#"{"a": [1, 2, {"b": 3"#,
        r#"{"a":[1,2,{"b":3}]}"#,
    );
}

// ── 13. Markdown code fences ────────────────────────────────

#[test]
fn markdown_json_fence() {
    ok("```json\n{\"a\": 1}\n```", r#"{"a":1}"#);
}

#[test]
fn markdown_plain_fence() {
    ok("```\n{\"a\": 1}\n```", r#"{"a":1}"#);
}

// ── 14. Numbers ─────────────────────────────────────────────

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

// ── 15. MongoDB constructors ────────────────────────────────

#[test]
fn mongodb_object_id() {
    ok(
        r#"{"_id": ObjectId("123abc")}"#,
        r#"{"_id":"123abc"}"#,
    );
}

#[test]
fn mongodb_number_long() {
    ok(
        r#"{"count": NumberLong("42")}"#,
        r#"{"count":"42"}"#,
    );
}

// ── 16. JSONP ───────────────────────────────────────────────

#[test]
fn jsonp_callback() {
    ok(
        r#"callback({"a": 1})"#,
        r#"{"a":1}"#,
    );
}

#[test]
fn trailing_semicolon() {
    ok(r#"{"a": 1};"#, r#"{"a":1}"#);
}

// ── 17. Ellipsis ────────────────────────────────────────────

#[test]
fn ellipsis_in_array() {
    ok("[1, 2, ...]", "[1,2]");
}

#[test]
fn ellipsis_in_object() {
    ok(r#"{"a": 1, ...}"#, r#"{"a":1}"#);
}

// ── 18. BOM handling ────────────────────────────────────────

#[test]
fn bom_prefix() {
    ok("\u{FEFF}{\"a\": 1}", r#"{"a":1}"#);
}

// ── 19. Special whitespace ──────────────────────────────────

#[test]
fn non_breaking_space() {
    ok("{\u{00A0}\"a\": 1}", r#"{"a":1}"#);
}

// ── 20. NDJSON ──────────────────────────────────────────────

#[test]
fn ndjson_two_objects() {
    ok(
        "{\"a\":1}\n{\"b\":2}",
        "[\n{\"a\":1},\n{\"b\":2}\n]",
    );
}

// ── 21. Error cases ─────────────────────────────────────────

#[test]
fn empty_input() {
    err("");
}

#[test]
fn whitespace_only() {
    err("   ");
}

// ── 22. Complex / mixed repairs ─────────────────────────────

#[test]
fn mixed_repairs() {
    ok(
        "{'name': 'John', age: 30, 'active': True,}",
        r#"{"name":"John","age":30,"active":true}"#,
    );
}

#[test]
fn deeply_nested_with_issues() {
    ok(
        "{a: [1, {b: 'hello' c: True},]}",
        r#"{"a":[1,{"b":"hello","c":true}]}"#,
    );
}

#[test]
fn object_with_comments_and_trailing_commas() {
    ok(
        r#"{
            // name
            "name": "John",
            /* age */
            "age": 30,
        }"#,
        r#"{"name":"John","age":30}"#,
    );
}
