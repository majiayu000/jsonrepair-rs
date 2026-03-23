use std::fmt;

/// Error returned when JSON cannot be repaired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRepairError {
    /// Human-readable message.
    pub message: String,
    /// Byte offset in the original input where the error was detected.
    pub position: usize,
}

impl JsonRepairError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for JsonRepairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON repair error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for JsonRepairError {}
