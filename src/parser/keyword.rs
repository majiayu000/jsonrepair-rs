use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    /// Parse JSON/Python/JS keywords or fall back to unquoted string.
    /// Compares directly on char slice without allocating.
    pub(super) fn parse_keyword_or_unquoted(&mut self) -> Result<bool> {
        let start = self.pos;
        if !self.peek().is_some_and(chars::is_identifier_start) {
            return Ok(false);
        }
        while self.peek().is_some_and(chars::is_identifier_char) {
            self.pos += 1;
        }

        let replacement = match self.keyword_replacement(start, self.pos) {
            Some(value) => value,
            None => {
                self.pos = start;
                return self.parse_unquoted_string(false);
            }
        };

        self.output.push_str(replacement);
        Ok(true)
    }

    /// Parse signed special values like `-Infinity` and `+NaN`.
    pub(super) fn parse_signed_keyword(&mut self) -> Result<bool> {
        let start = self.pos;
        if !matches!(self.peek(), Some('+') | Some('-')) {
            return Ok(false);
        }

        self.pos += 1;
        while self.peek().is_some_and(|c| matches!(c, ' ' | '\t' | '\r')) {
            self.pos += 1;
        }
        if !self.peek().is_some_and(chars::is_identifier_start) {
            self.pos = start;
            return Ok(false);
        }

        let token_start = self.pos;
        while self.peek().is_some_and(chars::is_identifier_char) {
            self.pos += 1;
        }

        if !self.at_token_boundary() {
            self.pos = start;
            return Ok(false);
        }

        if self.is_case_insensitive_keyword(token_start, self.pos, "nan")
            || self.is_case_insensitive_keyword(token_start, self.pos, "infinity")
        {
            self.output.push_str("null");
            return Ok(true);
        }

        self.pos = start;
        Ok(false)
    }

    fn keyword_replacement(&self, start: usize, end: usize) -> Option<&'static str> {
        if self.is_case_insensitive_keyword(start, end, "true") {
            return Some("true");
        }
        if self.is_case_insensitive_keyword(start, end, "false") {
            return Some("false");
        }
        if self.is_case_insensitive_keyword(start, end, "null")
            || self.is_case_insensitive_keyword(start, end, "none")
            || self.is_case_insensitive_keyword(start, end, "undefined")
            || self.is_case_insensitive_keyword(start, end, "nan")
            || self.is_case_insensitive_keyword(start, end, "infinity")
        {
            return Some("null");
        }
        None
    }

    fn is_case_insensitive_keyword(&self, start: usize, end: usize, keyword: &str) -> bool {
        if end - start != keyword.len() {
            return false;
        }

        keyword
            .chars()
            .enumerate()
            .all(|(offset, expected)| self.chars[start + offset].eq_ignore_ascii_case(&expected))
    }

    fn at_token_boundary(&self) -> bool {
        match self.peek() {
            None => true,
            Some(c) => chars::is_delimiter(c) || chars::is_whitespace(c),
        }
    }
}
