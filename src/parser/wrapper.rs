use crate::chars;
use crate::error::JsonRepairErrorKind;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    fn is_known_wrapper_function(&self, start: usize, end: usize) -> bool {
        self.slice_starts_with_ignore_ascii_case(start, end, "callback")
            || self.slice_eq_ignore_ascii_case(start, end, "cb")
            || self.slice_starts_with_ignore_ascii_case(start, end, "jsonp")
            || self.slice_starts_with(start, end, "jQuery")
            || self.slice_eq_ignore_ascii_case(start, end, "ObjectId")
            || self.slice_eq_ignore_ascii_case(start, end, "NumberLong")
            || self.slice_eq_ignore_ascii_case(start, end, "NumberInt")
            || self.slice_eq_ignore_ascii_case(start, end, "NumberDecimal")
            || self.slice_eq_ignore_ascii_case(start, end, "ISODate")
    }

    #[inline(always)]
    pub(super) fn maybe_known_wrapper_start(&self) -> bool {
        match self.peek() {
            Some(c) => matches!(c, 'c' | 'C' | 'j' | 'J' | 'o' | 'O' | 'n' | 'N' | 'i' | 'I'),
            None => false,
        }
    }

    /// Parse known JSONP/Mongo wrappers:
    /// - callback(...)
    /// - ObjectId(...)
    /// - new ObjectId(...)
    pub(super) fn parse_known_wrapper_call(&mut self) -> Result<bool> {
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
        if self.slice_eq_ignore_ascii_case(start, name_end, "new") {
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

        self.enter_container()?;
        self.pos = cursor + 1;
        let output_start = self.output.len();
        self.parse_whitespace_and_comments();
        let value_parsed = if self.peek() == Some(')') {
            self.output.push_str("null");
            true
        } else {
            let value_parsed = self.parse_value(true)?;
            if !value_parsed {
                self.output.push_str("null");
            }
            value_parsed
        };

        if self.peek() == Some(',') {
            // Property recovery applies only when the wrapper close is missing.
            let closed_wrapper = self.in_object && self.wrapper_has_closing_parenthesis(cursor)?;
            let mut multiple_arguments = false;
            while self.peek() == Some(',')
                && (closed_wrapper || !self.comma_starts_object_property()?)
            {
                if !value_parsed {
                    return Err(self.error_char_kind(
                        "Unexpected character",
                        JsonRepairErrorKind::UnexpectedCharacter,
                    ));
                }
                if !multiple_arguments {
                    self.output.insert(output_start, '[');
                    multiple_arguments = true;
                }
                self.parse_char(',');
                self.parse_whitespace_and_comments();
                if self.at_end() {
                    return Err(self.error_kind(
                        "Unexpected end of json string",
                        JsonRepairErrorKind::UnexpectedEnd,
                    ));
                }
                if self.peek() == Some(')') || !self.parse_value(true)? {
                    return Err(self.error_char_kind(
                        "Unexpected character",
                        JsonRepairErrorKind::UnexpectedCharacter,
                    ));
                }
            }
            if multiple_arguments {
                let ndjson_boundary = self.peek().is_some_and(chars::is_start_of_value)
                    && self.output_ends_with_comma_or_newline();
                if !matches!(self.peek(), Some(',' | ')' | '}' | ']'))
                    && !self.at_end()
                    && !ndjson_boundary
                {
                    return Err(self.error_char_kind(
                        "Unexpected character",
                        JsonRepairErrorKind::UnexpectedCharacter,
                    ));
                }
                if ndjson_boundary {
                    self.insert_before_last_whitespace("]");
                } else {
                    self.output.push(']');
                }
            }
        }

        if self.peek() == Some(')') {
            self.pos += 1;
            if self.peek() == Some(';') {
                self.pos += 1;
            }
        }

        self.leave_container();
        Ok(true)
    }

    fn wrapper_has_closing_parenthesis(&mut self, start: usize) -> Result<bool> {
        struct KeyContext {
            is_object: bool,
            expects_key: bool,
        }

        let mut cursor = start + 1;
        let mut parentheses = 1usize;
        let mut containers = 0usize;
        let mut key_contexts: Vec<KeyContext> = Vec::new();
        let mut in_url = false;
        while let Some(c) = self.peek_at(cursor) {
            if in_url && !chars::is_url_char(c) {
                in_url = false;
            }
            if c == '/'
                && self.peek_at(cursor.saturating_sub(1)) == Some(':')
                && self.ends_with_url_scheme(start + 1, cursor)
            {
                in_url = true;
            }
            let is_object_key = key_contexts
                .last()
                .is_some_and(|context| context.is_object && context.expects_key);
            if !in_url
                && (chars::is_quote(c)
                    || chars::is_identifier_start(c)
                    || (self.options.preserve_comment_markers
                        && (c == '#'
                            || (c == '/' && matches!(self.peek_at(cursor + 1), Some('/' | '*'))))))
            {
                if chars::is_identifier_start(c) && !is_object_key {
                    let mut name_end = cursor;
                    while self
                        .peek_at(name_end)
                        .is_some_and(chars::is_identifier_char)
                    {
                        name_end += 1;
                    }
                    let mut after_name = name_end;
                    while self.peek_at(after_name).is_some_and(chars::is_whitespace) {
                        after_name += 1;
                    }
                    // Leave actual wrapper parentheses to the structural scan.
                    if self.slice_eq_ignore_ascii_case(cursor, name_end, "new")
                        || (self.peek_at(after_name) == Some('(')
                            && self.is_known_wrapper_function(cursor, name_end))
                    {
                        cursor = name_end;
                        continue;
                    }
                }
                let input_start = self.pos;
                let output_start = self.output.len();
                let depth_start = self.depth;
                self.pos = cursor;
                // Reuse repair boundaries, including missing quotes and embedded '#'.
                let parsed = if chars::is_quote(c) {
                    self.parse_string(containers == 0)
                } else if is_object_key {
                    // A nested object's key must stop before its value. This
                    // lets a following '//' value use the same URL boundaries
                    // as normal parsing, while wrapper/array values like a:b
                    // keep their existing unquoted-value semantics.
                    self.parse_unquoted_string(true, false)
                } else if chars::is_identifier_start(c) {
                    self.parse_keyword_or_unquoted(containers == 0)
                } else {
                    self.parse_unquoted_string(false, containers == 0)
                };
                cursor = self.pos;
                self.pos = input_start;
                self.output.truncate(output_start);
                self.depth = depth_start;
                parsed?;
                if is_object_key {
                    if let Some(context) = key_contexts.last_mut() {
                        context.expects_key = false;
                    }
                }
                continue;
            }
            if !self.options.preserve_comment_markers
                && !in_url
                && c == '/'
                && self.peek_at(cursor + 1) == Some('*')
            {
                cursor += 2;
                while self.peek_at(cursor).is_some() && !self.matches_at(cursor, "*/") {
                    cursor += 1;
                }
                cursor += 2;
                continue;
            }
            if !self.options.preserve_comment_markers
                && !in_url
                && ((c == '/' && self.peek_at(cursor + 1) == Some('/')) || c == '#')
            {
                while self.peek_at(cursor).is_some_and(|ch| ch != '\n') {
                    cursor += 1;
                }
                continue;
            }
            if !in_url && c == '/' {
                let mut regex_cursor = cursor + 1;
                let mut escaped = false;
                let mut regex_end = None;
                while let Some(regex_char) = self.peek_at(regex_cursor) {
                    if matches!(regex_char, '\n' | '\r') {
                        break;
                    }
                    regex_cursor += 1;
                    // Match the existing regex parser's unescaped-slash boundary.
                    if regex_char == '/' && !escaped {
                        regex_end = Some(regex_cursor);
                        break;
                    }
                    escaped = regex_char == '\\' && !escaped;
                }
                if let Some(end) = regex_end {
                    cursor = end;
                    while self
                        .peek_at(cursor)
                        .is_some_and(|ch| ch.is_ascii_alphabetic())
                    {
                        cursor += 1;
                    }
                    continue;
                }
            }
            match c {
                '{' | '[' => {
                    containers += 1;
                    if self.options.preserve_comment_markers {
                        key_contexts.push(KeyContext {
                            is_object: c == '{',
                            expects_key: true,
                        });
                    }
                }
                '}' | ']' if containers == 0 => return Ok(false),
                '}' | ']' => {
                    containers -= 1;
                    key_contexts.pop();
                }
                ',' => {
                    if let Some(context) = key_contexts.last_mut() {
                        context.expects_key = true;
                    }
                }
                ':' | '=' => {
                    // Numeric/symbol keys can reach their separator through
                    // the raw scan without entering the string parser above.
                    if let Some(context) = key_contexts.last_mut() {
                        context.expects_key = false;
                    }
                }
                '(' if containers == 0 => parentheses += 1,
                ')' if containers == 0 => {
                    parentheses -= 1;
                    if parentheses == 0 {
                        return Ok(true);
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
        Ok(false)
    }

    fn comma_starts_object_property(&mut self) -> Result<bool> {
        if !self.in_object {
            return Ok(false);
        }
        let input_start = self.pos;
        let output_start = self.output.len();
        self.pos += 1;
        self.parse_whitespace_and_comments();
        let key_start = self.pos;
        // Reuse the key parsers without consuming input or output, including on errors.
        let property = self
            .parse_key_string()
            .and_then(|parsed| {
                if parsed {
                    Ok(true)
                } else {
                    self.parse_unquoted_string(true, false)
                }
            })
            .map(|parsed| {
                self.parse_whitespace_and_comments();
                parsed
                    && matches!(self.peek(), Some(':' | '='))
                    && !(self.peek_at(self.pos + 1) == Some('/')
                        && self.ends_with_url_scheme(key_start, self.pos + 1))
            });
        self.pos = input_start;
        self.output.truncate(output_start);
        property
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

    fn slice_eq_ignore_ascii_case(&self, start: usize, end: usize, text: &str) -> bool {
        if end - start != text.len() {
            return false;
        }
        text.chars()
            .enumerate()
            .all(|(i, c)| self.chars[start + i].eq_ignore_ascii_case(&c))
    }

    fn slice_starts_with_ignore_ascii_case(&self, start: usize, end: usize, prefix: &str) -> bool {
        let prefix_len = prefix.len();
        if end - start < prefix_len {
            return false;
        }
        prefix
            .chars()
            .enumerate()
            .all(|(i, c)| self.chars[start + i].eq_ignore_ascii_case(&c))
    }
}
