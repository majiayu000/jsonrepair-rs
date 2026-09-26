#![cfg(feature = "serde")]

use jsonrepair_rs::{
    jsonrepair_parse, jsonrepair_parse_with_options, jsonrepair_value,
    jsonrepair_value_with_options, jsonrepair_value_with_schema, JsonRepairErrorKind,
    JsonRepairParseError, RepairOptions,
};
use serde_json::json;

#[test]
fn repairs_and_returns_value() {
    let value = jsonrepair_value("{name: 'Ada', active: True, attempts: [1,2,],}").unwrap();

    assert_eq!(value["name"], "Ada");
    assert_eq!(value["active"], true);
    assert_eq!(value["attempts"][1], 2);
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct User {
    name: String,
    active: bool,
}

#[test]
fn repairs_and_deserializes_target_type() {
    let user: User = jsonrepair_parse("{name: 'Ada', active: True}").unwrap();

    assert_eq!(
        user,
        User {
            name: "Ada".to_string(),
            active: true,
        }
    );
}

#[test]
fn preserves_repair_errors() {
    let err = jsonrepair_value(r#""\u00""#).unwrap_err();

    assert!(matches!(err, JsonRepairParseError::Repair(_)));
}

#[test]
fn strict_options_apply_to_serde_helpers() {
    let value = jsonrepair_value_with_options(
        r#"{"name": "Ada", "active": true}"#,
        RepairOptions::strict(),
    )
    .unwrap();
    assert_eq!(value["name"], "Ada");

    let err = jsonrepair_parse_with_options::<User>(
        "{name: 'Ada', active: True}",
        RepairOptions::strict(),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        JsonRepairParseError::Repair(repair_err)
            if repair_err.kind == JsonRepairErrorKind::StrictModeViolation
    ));
}

#[test]
fn corrects_nested_tool_arguments_from_valid_json() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer" },
            "active": { "type": "boolean" },
            "ratio": { "type": "number" },
            "tags": { "type": "array", "items": { "type": "string", "enum": ["Rust", "Go"] } },
            "nested": { "type": "object", "properties": { "level": { "type": "integer" } } }
        }
    });
    let input = r#"{"count":"5","active":"TRUE","ratio":"1.25","tags":"rust","nested":{"level":"2"},"extra":"4"}"#;

    let corrected = jsonrepair_value_with_schema(input, &schema).unwrap();

    assert_eq!(
        corrected,
        json!({
            "count": 5,
            "active": true,
            "ratio": 1.25,
            "tags": ["Rust"],
            "nested": { "level": 2 },
            "extra": "4"
        })
    );
}

#[test]
fn leaves_ambiguous_or_invalid_values_unchanged() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer" },
            "active": { "type": "boolean" },
            "choice": { "type": "string", "enum": ["Foo", "fOO"] },
            "tags": { "type": "array", "items": { "type": "string" } }
        }
    });
    let input = r#"{"count":"5.5","active":"yes","choice":"FOO","tags":null}"#;

    assert_eq!(
        jsonrepair_value_with_schema(input, &schema).unwrap(),
        serde_json::from_str::<serde_json::Value>(input).unwrap()
    );
}

#[test]
fn repairs_syntax_then_corrects_each_array_item() {
    let schema = json!({
        "type": "array",
        "items": { "type": "object", "properties": { "count": { "type": "integer" } } }
    });

    assert_eq!(
        jsonrepair_value_with_schema("[{count: '1'}, {count: '2',}]", &schema).unwrap(),
        json!([{ "count": 1 }, { "count": 2 }])
    );
}

#[test]
fn schema_correction_preserves_repair_errors() {
    let err = jsonrepair_value_with_schema(r#""\u00""#, &json!({ "type": "string" })).unwrap_err();

    assert!(matches!(err, JsonRepairParseError::Repair(_)));
}
