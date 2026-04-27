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

pub use error::{JsonRepairError, JsonRepairErrorKind};

/// Error returned by writer-based repair helpers.
#[derive(Debug)]
#[non_exhaustive]
pub enum JsonRepairWriteError {
    /// The input could not be repaired safely.
    Repair(JsonRepairError),
    /// The repaired JSON could not be written to the destination.
    Write(std::io::Error),
}

impl std::fmt::Display for JsonRepairWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repair(err) => err.fmt(f),
            Self::Write(err) => write!(f, "failed to write repaired JSON: {err}"),
        }
    }
}

impl std::error::Error for JsonRepairWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Repair(err) => Some(err),
            Self::Write(err) => Some(err),
        }
    }
}

impl From<JsonRepairError> for JsonRepairWriteError {
    fn from(err: JsonRepairError) -> Self {
        Self::Repair(err)
    }
}

impl From<std::io::Error> for JsonRepairWriteError {
    fn from(err: std::io::Error) -> Self {
        Self::Write(err)
    }
}

/// Repair a broken JSON string and write the valid JSON to a writer.
///
/// This is a writer convenience API. It repairs the input first, then writes
/// the repaired JSON bytes to the provided [`std::io::Write`] destination.
pub fn jsonrepair_to_writer<W>(input: &str, writer: &mut W) -> Result<(), JsonRepairWriteError>
where
    W: std::io::Write + ?Sized,
{
    let repaired = jsonrepair(input)?;
    writer.write_all(repaired.as_bytes())?;
    Ok(())
}

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
