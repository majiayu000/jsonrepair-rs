use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    /// Parse a string value. If `is_key` is true, context is an object key.
    pub(super) fn parse_string(&mut self, _is_key: bool) -> Result<bool> {
        let quote = match self.peek() {
            Some(c) if chars::is_quote(c) => c,
            _ => return Ok(false),
        };
        self.pos += 1;
        let is_double = chars::is_double_quote_like(quote);
        let close_fn: fn(char) -> bool = if is_double {
            chars::is_double_quote_like
        } else {
            chars::is_single_quote_like
        };
        self.output.push('"');
        self.parse_string_body(close_fn)?;
        Ok(true)
    }

    fn parse_string_body(&mut self, close_fn: fn(char) -> bool) -> Result<()> {
        loop {
            match self.peek() {
                None => {
                    self.output.push('"');
                    return Ok(());
                }
                Some(c) if close_fn(c) => {
                    self.pos += 1;
                    if self.try_string_concat(close_fn) {
                        continue;
                    }
                    self.output.push('"');
                    return Ok(());
                }
                Some('\\') => self.parse_escape()?,
                Some(c) => {
                    self.pos += 1;
                    self.push_string_char(c);
                }
            }
        }
    }

    fn try_string_concat(&mut self, close_fn: fn(char) -> bool) -> bool {
        let ws_start = self.output.len();
        self.parse_whitespace_and_comments();
        if self.peek() == Some('+') {
            let save_pos = self.pos;
            self.pos += 1;
            self.parse_whitespace_and_comments();
            if self.peek().is_some_and(|c| chars::is_quote(c)) {
                let nq = self.chars[self.pos];
                self.pos += 1;
                // Remove whitespace that was added between strings
                self.output.truncate(ws_start);
                let _ = close_fn; // next segment uses same close logic
                let nd = chars::is_double_quote_like(nq);
                let _ncf: fn(char) -> bool = if nd {
                    chars::is_double_quote_like
                } else {
                    chars::is_single_quote_like
                };
                return true;
            }
            // Not a concat — restore
            self.output.truncate(ws_start);
            self.pos = save_pos;
        } else {
            // Not a + — restore whitespace position tracking
            // The whitespace is fine to keep (it's after the string)
        }
        false
    }

    fn parse_escape(&mut self) -> Result<()> {
        self.pos += 1; // skip backslash
        match self.peek() {
            None => {
                self.output.push('"');
            }
            Some(esc) => {
                self.pos += 1;
                match esc {
                    '"' | '\\' | '/' => {
                        self.output.push('\\');
                        self.output.push(esc);
                    }
                    'n' => self.output.push_str("\\n"),
                    'r' => self.output.push_str("\\r"),
                    't' => self.output.push_str("\\t"),
                    'b' => self.output.push_str("\\b"),
                    'f' => self.output.push_str("\\f"),
                    'u' => self.parse_unicode_escape(),
                    '\'' => self.output.push('\''),
                    '\n' | '\r' => { /* line continuation — skip */ }
                    _ => self.output.push(esc),
                }
            }
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self) {
        self.output.push_str("\\u");
        let mut count = 0;
        while count < 4 {
            if let Some(h) = self.peek() {
                if chars::is_hex(h) {
                    self.output.push(h);
                    self.pos += 1;
                    count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        while count < 4 {
            self.output.push('0');
            count += 1;
        }
    }

    fn push_string_char(&mut self, c: char) {
        match c {
            '"' => self.output.push_str("\\\""),
            '\n' => self.output.push_str("\\n"),
            '\r' => self.output.push_str("\\r"),
            '\t' => self.output.push_str("\\t"),
            '\x08' => self.output.push_str("\\b"),
            '\x0C' => self.output.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                self.output.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => self.output.push(c),
        }
    }

    /// Parse an unquoted key (identifier style: alphanumeric, _, $, -).
    pub(super) fn parse_unquoted_key(&mut self) -> Result<bool> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if chars::is_identifier_char(c) || c == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        // Also accept digits as keys
        if self.pos == start {
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if self.pos == start {
            return Ok(false);
        }
        let key: String = self.chars[start..self.pos].iter().collect();
        self.output.push('"');
        self.output.push_str(&key);
        self.output.push('"');
        Ok(true)
    }

    /// Parse unquoted string value (multi-word, until delimiter).
    #[allow(dead_code)] // Used when full JS test suite is ported
    pub(super) fn parse_unquoted_string(&mut self) -> Result<bool> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if chars::is_unquoted_string_char(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Ok(false);
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        let trimmed = s.trim_end();
        // Put back trailing whitespace
        let ws_len = s.len() - trimmed.len();
        self.pos -= ws_len;
        self.output.push('"');
        self.output.push_str(trimmed);
        self.output.push('"');
        Ok(true)
    }
}
