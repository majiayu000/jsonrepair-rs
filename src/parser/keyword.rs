use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_keyword_or_unquoted(&mut self) -> Result<bool> {
        let start = self.pos;
        if !self.peek().is_some_and(chars::is_identifier_start) {
            return Ok(false);
        }
        while self.peek().is_some_and(chars::is_identifier_char) {
            self.pos += 1;
        }

        let word: String = self.chars[start..self.pos].iter().collect();

        match word.as_str() {
            "true" | "false" | "null" => {
                self.output.push_str(&word);
            }
            "True" => self.output.push_str("true"),
            "False" => self.output.push_str("false"),
            "None" => self.output.push_str("null"),
            "undefined" => self.output.push_str("null"),
            "NaN" => self.output.push_str("null"),
            "Infinity" => self.output.push_str("null"),
            _ => {
                self.pos = start;
                return self.parse_unquoted_string(false);
            }
        }
        Ok(true)
    }
}
