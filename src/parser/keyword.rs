use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    /// Parse JSON/Python/JS keywords or fall back to unquoted string.
    /// Compares directly on char slice — no String allocation.
    pub(super) fn parse_keyword_or_unquoted(&mut self) -> Result<bool> {
        let start = self.pos;
        if !self.peek().is_some_and(chars::is_identifier_start) {
            return Ok(false);
        }
        while self.peek().is_some_and(chars::is_identifier_char) {
            self.pos += 1;
        }

        let replacement = if self.slice_eq(start, self.pos, "true")
            || self.slice_eq(start, self.pos, "True")
        {
            "true"
        } else if self.slice_eq(start, self.pos, "false") || self.slice_eq(start, self.pos, "False")
        {
            "false"
        } else if self.slice_eq(start, self.pos, "null")
            || self.slice_eq(start, self.pos, "None")
            || self.slice_eq(start, self.pos, "undefined")
            || self.slice_eq(start, self.pos, "NaN")
            || self.slice_eq(start, self.pos, "Infinity")
        {
            "null"
        } else {
            self.pos = start;
            return self.parse_unquoted_string(false);
        };

        self.output.push_str(replacement);
        Ok(true)
    }
}
