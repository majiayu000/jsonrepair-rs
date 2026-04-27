//! # jsonrepair-rs
//!
//! Repair broken JSON — fix quotes, commas, comments, trailing content,
//! and 30+ other issues commonly found in LLM outputs.
//!
//! Port of the JavaScript [jsonrepair](https://github.com/josdejong/jsonrepair) library.
//!
//! ## Usage
//!
//! ```
//! use jsonrepair_rs::jsonrepair;
//!
//! // Fix single quotes (whitespace preserved)
//! let result = jsonrepair("{'name': 'John'}").unwrap();
//! assert_eq!(result, r#"{"name": "John"}"#);
//!
//! // Fix trailing commas
//! let result = jsonrepair(r#"{"a": 1, "b": 2,}"#).unwrap();
//! assert_eq!(result, r#"{"a": 1, "b": 2}"#);
//!
//! // Convert Python keywords
//! let result = jsonrepair(r#"{"flag": True, "value": None}"#).unwrap();
//! assert_eq!(result, r#"{"flag": true, "value": null}"#);
//! ```

mod chars;
mod error;
mod parser;

#[cfg(feature = "serde")]
pub use error::JsonRepairParseError;
pub use error::{JsonRepairError, JsonRepairErrorKind};

/// Repair a broken JSON string, returning valid JSON.
///
/// Handles 30+ categories of issues including:
/// - Single/curly quotes → double quotes
/// - Trailing/missing commas
/// - Comments (`//`, `/* */`, `#`)
/// - Python keywords (`True`, `False`, `None`)
/// - JavaScript keywords (`undefined`, `NaN`, `Infinity`)
/// - Markdown code fences
/// - JSONP wrappers
/// - Unquoted keys and strings
/// - Truncated JSON (auto-closes brackets)
/// - String concatenation (`"a" + "b"`)
/// - Invalid escape sequences
/// - Leading zeros, truncated numbers
/// - MongoDB constructors (`ObjectId(...)`)
/// - NDJSON (newline-delimited JSON)
/// - Ellipsis operators (`...`)
///
/// Returns `Err(JsonRepairError)` if the input cannot be repaired.
pub fn jsonrepair(input: &str) -> Result<String, JsonRepairError> {
    let repairer = parser::JsonRepairer::new(input);
    repairer.repair()
}

/// Repair a broken JSON string and parse it into [`serde_json::Value`].
///
/// This helper is available with the `serde` feature.
#[cfg(feature = "serde")]
pub fn jsonrepair_value(input: &str) -> Result<serde_json::Value, JsonRepairParseError> {
    jsonrepair_parse(input)
}

/// Repair a broken JSON string and deserialize it into the requested type.
///
/// This helper is available with the `serde` feature.
#[cfg(feature = "serde")]
pub fn jsonrepair_parse<T>(input: &str) -> Result<T, JsonRepairParseError>
where
    T: serde::de::DeserializeOwned,
{
    let repaired = jsonrepair(input)?;
    serde_json::from_str(&repaired).map_err(JsonRepairParseError::from)
}
