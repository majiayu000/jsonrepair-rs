use crate::chars;
use crate::error::JsonRepairErrorKind;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    /// Parse a quoted string value.
    pub(super) fn parse_string(&mut self) -> Result<bool> {
        self.parse_string_internal(false, None)
    }

    fn parse_string_internal(
        &mut self,
        stop_at_delimiter: bool,
        stop_at_index: Option<usize>,
    ) -> Result<bool> {
        let skip_escape_chars = self.peek() == Some('\\');
        if skip_escape_chars {
            // repair escaped string start: \"foo\"
            self.pos += 1;
        }

        let quote = match self.peek() {
            Some(c) if chars::is_quote(c) => c,
            _ => return Ok(false),
        };
        let is_end_quote: fn(char) -> bool = if chars::is_double_quote(quote) {
            chars::is_double_quote
        } else if chars::is_single_quote(quote) {
            chars::is_single_quote
        } else if chars::is_single_quote_like(quote) {
            chars::is_single_quote_like
        } else {
            chars::is_double_quote_like
        };

        let input_start = self.pos;
        let output_start = self.output.len();
        self.output.push('"');
        self.pos += 1;

        loop {
            if self.at_end() {
                if !stop_at_delimiter
                    && self
                        .pos
                        .checked_sub(1)
                        .and_then(|idx| self.prev_non_whitespace_index(idx))
                        .is_some_and(|idx| chars::is_delimiter(self.chars[idx]))
                {
                    // Retry in conservative mode when we ended after a delimiter.
                    self.pos = input_start;
                    self.output.truncate(output_start);
                    return self.parse_string_internal(true, None);
                }

                self.insert_before_last_output_whitespace(output_start + 1, "\"");
                return Ok(true);
            }

            if stop_at_index.is_some_and(|idx| self.pos == idx) {
                self.insert_before_last_output_whitespace(output_start + 1, "\"");
                return Ok(true);
            }

            let c = self.chars[self.pos];
            if is_end_quote(c) {
                let quote_pos = self.pos;
                let quote_output_pos = self.output.len();
                self.output.push('"');
                self.pos += 1;

                self.parse_whitespace_and_comments_with_newline(false);
                let next = self.peek();
                if stop_at_delimiter
                    || next.is_none()
                    || next.is_some_and(|ch| {
                        chars::is_delimiter(ch) || chars::is_quote(ch) || chars::is_digit(ch)
                    })
                {
                    self.parse_concatenated_string()?;
                    return Ok(true);
                }

                let prev_non_ws = quote_pos
                    .checked_sub(1)
                    .and_then(|idx| self.prev_non_whitespace_index(idx));
                let prev_char = prev_non_ws.and_then(|idx| self.peek_at(idx));

                if prev_char == Some(',') {
                    // {"a":"b,c,"d":"e"} -> stop at comma before quote.
                    self.pos = input_start;
                    self.output.truncate(output_start);
                    return self.parse_string_internal(false, prev_non_ws);
                }

                if prev_char.is_some_and(chars::is_delimiter) {
                    // End quote likely missing earlier.
                    self.pos = input_start;
                    self.output.truncate(output_start);
                    return self.parse_string_internal(true, None);
                }

                // Not a real closing quote: continue, escaping this quote.
                self.output.truncate(quote_output_pos);
                self.output.push_str("\\\"");
                self.pos = quote_pos + 1;
            } else if stop_at_delimiter && chars::is_unquoted_string_delimiter(c) {
                // URL like "https://..." should not stop at '/'.
                if self.pos > input_start + 1
                    && self.peek_at(self.pos.saturating_sub(1)) == Some(':')
                    && self.looks_like_url_start(input_start + 1, self.pos)
                {
                    while self.peek().is_some_and(chars::is_url_char) {
                        self.output.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                }

                self.insert_before_last_output_whitespace(output_start + 1, "\"");
                self.parse_concatenated_string()?;
                return Ok(true);
            } else if c == '\\' {
                self.parse_string_escape()?;
            } else {
                self.parse_string_char(c)?;
            }

            if skip_escape_chars {
                // Repair escaped outer string wrappers: consume \ before a quote.
                if self.peek() == Some('\\') {
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_string_escape(&mut self) -> Result<()> {
        let backslash_pos = self.pos;
        self.pos += 1; // skip '\'
        let esc = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(());
            }
        };

        match esc {
            '"' | '\\' | '/' => {
                self.output.push('\\');
                self.output.push(esc);
                self.pos += 1;
            }
            'b' | 'f' | 'n' | 'r' | 't' => {
                self.output.push('\\');
                self.output.push(esc);
                self.pos += 1;
            }
            'u' => {
                let mut digits = 0;
                while digits < 4
                    && self
                        .peek_at(self.pos + 1 + digits)
                        .is_some_and(chars::is_hex)
                {
                    digits += 1;
                }

                if digits == 4 {
                    self.output.push_str("\\u");
                    for i in 0..4 {
                        self.output.push(self.chars[self.pos + 1 + i]);
                    }
                    self.pos += 5; // 'u' + 4 hex digits
                } else if self.pos + 1 + digits >= self.chars.len() {
                    // Truncated unicode at end: end string here.
                    self.pos = self.chars.len();
                } else {
                    let end = (backslash_pos + 6).min(self.chars.len());
                    let snippet: String = self.chars[backslash_pos..end].iter().collect();
                    return Err(self.error_at_kind(
                        &format!("Invalid unicode character \"{snippet}\""),
                        backslash_pos,
                        JsonRepairErrorKind::InvalidUnicode,
                    ));
                }
            }
            '\'' => {
                // Keep a raw apostrophe when escaping single quote.
                self.output.push('\'');
                self.pos += 1;
            }
            '\n' | '\r' => {
                // Line continuation: drop it.
                self.pos += 1;
            }
            _ => {
                // Invalid escape: drop '\' and keep char.
                self.output.push(esc);
                self.pos += 1;
            }
        }
        Ok(())
    }

    fn parse_string_char(&mut self, c: char) -> Result<()> {
        if c >= '\u{0020}' && c != '"' && c != '\\' {
            self.output.push(c);
            self.pos += 1;
            return Ok(());
        }

        if c == '"' {
            self.output.push_str("\\\"");
            self.pos += 1;
            return Ok(());
        }

        match c {
            '\n' => self.output.push_str("\\n"),
            '\r' => self.output.push_str("\\r"),
            '\t' => self.output.push_str("\\t"),
            '\x08' => self.output.push_str("\\b"),
            '\x0C' => self.output.push_str("\\f"),
            _ => {
                if !chars::is_valid_string_character(c) {
                    return Err(self.error_kind(
                        &format!("Invalid character {:?}", c),
                        JsonRepairErrorKind::InvalidCharacter,
                    ));
                }
                self.output.push(c);
            }
        }

        self.pos += 1;
        Ok(())
    }

    fn parse_concatenated_string(&mut self) -> Result<bool> {
        let mut processed = false;

        self.parse_whitespace_and_comments();
        while self.peek() == Some('+') {
            processed = true;
            self.pos += 1;
            self.parse_whitespace_and_comments();

            // Remove end quote and any trailing whitespace/comments after it.
            if let Some(idx) = self.output.rfind('"') {
                self.output.truncate(idx);
            }
            let second_start = self.output.len();
            if self.parse_string()? {
                // Remove start quote from second string.
                if second_start < self.output.len() {
                    self.output.remove(second_start);
                }
            } else {
                // '+' not followed by a string.
                self.insert_before_last_whitespace("\"");
            }
        }

        Ok(processed)
    }

    /// Parse unquoted string values and function-call wrappers (MongoDB/JSONP).
    pub(super) fn parse_unquoted_string(&mut self, is_key: bool) -> Result<bool> {
        let start = self.pos;

        if self.peek().is_some_and(chars::is_identifier_start) && self.parse_known_wrapper_call()? {
            return Ok(true);
        }

        let mut parenthesis_depth = 0usize;
        while let Some(c) = self.peek() {
            if c == '(' {
                parenthesis_depth += 1;
                self.pos += 1;
                continue;
            }

            if c == ')' && parenthesis_depth > 0 {
                parenthesis_depth -= 1;
                self.pos += 1;
                continue;
            }

            if parenthesis_depth == 0
                && (chars::is_unquoted_string_delimiter(c)
                    || chars::is_quote(c)
                    || (is_key && c == ':'))
            {
                break;
            }
            self.pos += 1;
        }

        if self.pos > start
            && self.peek_at(self.pos.saturating_sub(1)) == Some(':')
            && self.looks_like_url_start(start, self.pos)
        {
            while self.peek().is_some_and(chars::is_url_char) {
                self.pos += 1;
            }
        }

        if self.pos == start {
            return Ok(false);
        }

        while self.pos > start && chars::is_whitespace(self.chars[self.pos - 1]) {
            self.pos -= 1;
        }

        // Compare directly on char slice — no String allocation.
        if !is_key && self.slice_eq(start, self.pos, "undefined") {
            self.output.push_str("null");
        } else {
            self.output.push('"');
            for i in start..self.pos {
                self.push_string_char(self.chars[i]);
            }
            self.output.push('"');
        }

        if self.peek().is_some_and(chars::is_quote) {
            // Missing start quote: consume dangling end quote.
            self.pos += 1;
        }

        Ok(true)
    }

    /// Check if chars starting at `start` look like a URL scheme (no allocation).
    fn looks_like_url_start(&self, start: usize, slash_idx: usize) -> bool {
        if self.peek_at(slash_idx) != Some('/') || self.peek_at(slash_idx + 1) != Some('/') {
            return false;
        }
        if slash_idx + 2 > self.chars.len() || start >= slash_idx + 2 {
            return false;
        }
        self.matches_at(start, "http://")
            || self.matches_at(start, "https://")
            || self.matches_at(start, "ftp://")
            || self.matches_at(start, "mailto://")
            || self.matches_at(start, "file://")
            || self.matches_at(start, "data://")
            || self.matches_at(start, "irc://")
    }

    fn is_known_wrapper_function(&self, start: usize, end: usize) -> bool {
        self.slice_starts_with(start, end, "callback")
            || self.slice_eq(start, end, "cb")
            || self.slice_starts_with(start, end, "jsonp")
            || self.slice_starts_with(start, end, "jQuery")
            || self.slice_eq(start, end, "ObjectId")
            || self.slice_eq(start, end, "NumberLong")
            || self.slice_eq(start, end, "NumberInt")
            || self.slice_eq(start, end, "NumberDecimal")
            || self.slice_eq(start, end, "ISODate")
    }

    /// Parse known JSONP/Mongo wrappers:
    /// - callback(...)
    /// - ObjectId(...)
    /// - new ObjectId(...)
    fn parse_known_wrapper_call(&mut self) -> Result<bool> {
        let start = self.pos;
        while self.peek().is_some_and(chars::is_identifier_char) {
            self.pos += 1;
        }

        let mut name_start = start;
        let mut name_end = self.pos;
        let mut cursor = self.pos;
        while self.peek_at(cursor).is_some_and(chars::is_whitespace) {
            cursor += 1;
        }

        // Support `new ObjectId("...")` style wrappers.
        if self.slice_eq(start, name_end, "new") {
            name_start = cursor;
            if !self.peek_at(cursor).is_some_and(chars::is_identifier_start) {
                self.pos = start;
                return Ok(false);
            }
            while self.peek_at(cursor).is_some_and(chars::is_identifier_char) {
                cursor += 1;
            }
            name_end = cursor;
            while self.peek_at(cursor).is_some_and(chars::is_whitespace) {
                cursor += 1;
            }
        }

        if self.peek_at(cursor) != Some('(')
            || !self.is_known_wrapper_function(name_start, name_end)
        {
            self.pos = start;
            return Ok(false);
        }

        self.pos = cursor + 1;
        self.parse_whitespace_and_comments();
        if self.peek() == Some(')') {
            self.output.push_str("null");
        } else {
            let value_parsed = self.parse_value()?;
            if !value_parsed {
                self.output.push_str("null");
            }
        }

        if self.peek() == Some(')') {
            self.pos += 1;
            if self.peek() == Some(';') {
                self.pos += 1;
            }
        }

        Ok(true)
    }

    fn slice_starts_with(&self, start: usize, end: usize, prefix: &str) -> bool {
        let prefix_len = prefix.len();
        if end - start < prefix_len {
            return false;
        }
        prefix
            .chars()
            .enumerate()
            .all(|(i, c)| self.chars[start + i] == c)
    }

    fn insert_before_last_output_whitespace(&mut self, start: usize, text: &str) {
        let bytes = self.output.as_bytes();
        let mut idx = bytes.len();
        while idx > start && matches!(bytes[idx - 1], b' ' | b'\n' | b'\r' | b'\t') {
            idx -= 1;
        }
        self.output.insert_str(idx, text);
    }
}
