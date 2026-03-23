use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_keyword_or_unquoted(&mut self) -> Result<bool> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if chars::is_identifier_char(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Ok(false);
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
            _ if self.is_mongodb_type(&word) => {
                self.parse_function_call()?;
            }
            _ => {
                return self.handle_unquoted_word(word);
            }
        }
        Ok(true)
    }

    fn is_mongodb_type(&self, word: &str) -> bool {
        matches!(
            word,
            "ObjectId" | "NumberLong" | "NumberInt" | "NumberDecimal" | "ISODate"
        )
    }

    fn parse_function_call(&mut self) -> Result<()> {
        self.parse_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.pos += 1;
            self.parse_whitespace_and_comments();
            if !self.parse_value()? {
                self.output.push_str("null");
            }
            self.parse_whitespace_and_comments();
            if self.peek() == Some(')') {
                self.pos += 1;
            }
        }
        Ok(())
    }

    fn handle_unquoted_word(&mut self, word: String) -> Result<bool> {
        // Check if followed by ( — JSONP function call
        self.parse_whitespace_and_comments();
        if self.peek() == Some('(') {
            self.pos += 1;
            self.parse_whitespace_and_comments();
            if !self.parse_value()? {
                self.output.push_str("null");
            }
            self.parse_whitespace_and_comments();
            if self.peek() == Some(')') {
                self.pos += 1;
            }
            return Ok(true);
        }

        // Unquoted string — collect remaining non-delimiter chars
        // (handles multi-word: `hello world`)
        let mut full = word;
        while let Some(c) = self.peek() {
            if chars::is_unquoted_string_char(c) {
                full.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        let trimmed = full.trim_end();
        let ws_len = full.len() - trimmed.len();
        self.pos -= ws_len;

        self.output.push('"');
        // Escape any double quotes inside
        for c in trimmed.chars() {
            if c == '"' {
                self.output.push_str("\\\"");
            } else {
                self.output.push(c);
            }
        }
        self.output.push('"');
        Ok(true)
    }
}
