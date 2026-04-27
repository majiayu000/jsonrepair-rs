#![cfg(feature = "serde")]

use jsonrepair_rs::{jsonrepair_parse, jsonrepair_value, JsonRepairParseError};

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
