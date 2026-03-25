use std::fmt;

/// Category of JSON repair error for programmatic handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JsonRepairErrorKind {
    /// Input ended unexpectedly.
    UnexpectedEnd,
    /// Encountered an unexpected character.
    UnexpectedCharacter,
    /// Invalid unicode escape sequence.
    InvalidUnicode,
    /// Expected an object key.
    ObjectKeyExpected,
    /// Expected a colon separator.
    ColonExpected,
    /// Maximum nesting depth exceeded.
    MaxDepthExceeded,
    /// Invalid control character in string.
    InvalidCharacter,
}

/// Error returned when JSON cannot be repaired.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct JsonRepairError {
    /// Human-readable message.
    pub message: String,
    /// Char offset in the original input where the error was detected.
    pub position: usize,
    /// Error category for programmatic handling.
    pub kind: JsonRepairErrorKind,
    /// 1-based line number (0 if unavailable).
    pub line: usize,
    /// 1-based column number (0 if unavailable).
    pub column: usize,
}

impl JsonRepairError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
            kind: JsonRepairErrorKind::UnexpectedCharacter,
            line: 0,
            column: 0,
        }
    }

    pub fn with_kind(
        message: impl Into<String>,
        position: usize,
        kind: JsonRepairErrorKind,
    ) -> Self {
        Self {
            message: message.into(),
            position,
            kind,
            line: 0,
            column: 0,
        }
    }

    pub(crate) fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.column = column;
        self
    }
}

impl fmt::Display for JsonRepairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(
                f,
                "JSON repair error at position {} (line {}, col {}): {}",
                self.position, self.line, self.column, self.message
            )
        } else {
            write!(
                f,
                "JSON repair error at position {}: {}",
                self.position, self.message
            )
        }
    }
}

impl std::error::Error for JsonRepairError {}
