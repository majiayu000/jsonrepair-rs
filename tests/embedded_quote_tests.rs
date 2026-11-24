use jsonrepair_rs::jsonrepair;

fn ok(input: &str, expected: &str) {
    let result = jsonrepair(input).unwrap_or_else(|e| panic!("repair failed for {input:?}: {e}"));
    assert_eq!(result, expected, "input: {input:?}");
}

// An unescaped quoted phrase that ends right at the end of a value must stay
// inside the value instead of becoming a new key or array element.

#[test]
fn embedded_quotes_at_value_end_in_object() {
    ok(
        r#"{"text":"跟进"25年12月""}"#,
        r#"{"text":"跟进\"25年12月\""}"#,
    );
    ok(
        r#"{"text":"跟进"25年12月"", "b":1}"#,
        r#"{"text":"跟进\"25年12月\"", "b":1}"#,
    );
    ok(r#"{"a":"he said "hi""}"#, r#"{"a":"he said \"hi\""}"#);
}

#[test]
fn embedded_quotes_at_value_end_in_array() {
    ok(r#"["a "quote""]"#, r#"["a \"quote\""]"#);
    ok(r#"["a "quote"", "b"]"#, r#"["a \"quote\"", "b"]"#);
    ok(r#"["x"y""]"#, r#"["x\"y\""]"#);
}

#[test]
fn embedded_quote_followed_by_digits_that_are_not_a_number() {
    ok(r#"{"a":"v"2b c"}"#, r#"{"a":"v\"2b c"}"#);
}

#[test]
fn quote_boundaries_that_still_end_the_string() {
    // A real number after the quote is the next value.
    ok(r#"["a"2]"#, r#"["a",2]"#);
    ok(r#"["a"2.5e3, 1]"#, r#"["a",2.5e3, 1]"#);
    ok(r#"{"a"1}"#, r#"{"a":1}"#);
    // Adjacent strings and empty strings keep their meaning.
    ok(r#"["a""b"]"#, r#"["a","b"]"#);
    ok(r#"["a",""]"#, r#"["a",""]"#);
    ok(r#"["a" ""]"#, r#"["a", ""]"#);
    // A balanced embedded phrase in the middle is unchanged.
    ok(r#"{"a":"x "y" z"}"#, r#"{"a":"x \"y\" z"}"#);
}
