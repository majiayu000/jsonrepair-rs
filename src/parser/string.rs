use crate::chars;
use crate::error::JsonRepairErrorKind;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    /// Parse a quoted string value.
    pub(super) fn parse_string(&mut self, is_wrapper_argument: bool) -> Result<bool> {
        self.parse_string_internal(false, None, is_wrapper_argument)
    }

    fn parse_string_internal(
        &mut self,
        stop_at_delimiter: bool,
        stop_at_index: Option<usize>,
        is_wrapper_argument: bool,
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
        let quote_kind = if chars::is_double_quote(quote) {
            0u8
        } else if chars::is_single_quote(quote) {
            1u8
        } else if chars::is_single_quote_like(quote) {
            2u8
        } else {
            3u8
        };

        let input_start = self.pos;
        let output_start = self.output.len();
        self.output.push('"');
        self.pos += 1;
        let mut parenthesis_depth = 0usize;
        let is_wrapper_close = |input: &[char], pos: usize, depth: usize| {
            if !is_wrapper_argument || input.get(pos) != Some(&')') || depth != 0 {
                return false;
            }
            // A colon still belongs to the string; enclosing boundaries end it.
            let next = input[pos + 1..]
                .iter()
                .copied()
                .find(|&ch| !chars::is_whitespace(ch) || matches!(ch, '\n' | '\r'));
            next.map_or(true, |ch| {
                (chars::is_delimiter(ch) && ch != ':') || ch == '#'
            })
        };
        let mut url_followed_by_content = false;

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
                    return self.parse_string_internal(true, None, is_wrapper_argument);
                }

                self.insert_before_last_output_whitespace(output_start + 1, "\"");
                return Ok(true);
            }

            if let Some(idx) = stop_at_index {
                if self.pos == idx {
                    self.insert_before_last_output_whitespace(output_start + 1, "\"");
                    return Ok(true);
                }
            }

            let c = self.chars[self.pos];
            let is_end_quote = match quote_kind {
                0 => chars::is_double_quote(c),
                1 => chars::is_single_quote(c),
                2 => chars::is_single_quote_like(c),
                _ => chars::is_double_quote_like(c),
            };
            if is_end_quote {
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
                    self.parse_concatenated_string(is_wrapper_argument)?;
                    return Ok(true);
                }

                let prev_non_ws = quote_pos
                    .checked_sub(1)
                    .and_then(|idx| self.prev_non_whitespace_index(idx));
                let prev_char = prev_non_ws.and_then(|idx| self.peek_at(idx));

                if prev_char == Some(',') && stop_at_index != prev_non_ws {
                    // {"a":"b,c,"d":"e"} -> stop at comma before quote.
                    self.pos = input_start;
                    self.output.truncate(output_start);
                    return self.parse_string_internal(false, prev_non_ws, is_wrapper_argument);
                }

                if prev_char.is_some_and(chars::is_delimiter) {
                    // End quote likely missing earlier.
                    self.pos = input_start;
                    self.output.truncate(output_start);
                    return self.parse_string_internal(true, None, is_wrapper_argument);
                }

                // Not a real closing quote: continue, escaping this quote.
                self.output.truncate(quote_output_pos);
                self.output.push_str("\\\"");
                self.pos = quote_pos + 1;
            } else if stop_at_delimiter
                && (chars::is_unquoted_string_delimiter(c)
                    || is_wrapper_close(&self.chars, self.pos, parenthesis_depth))
                && !(c == '/'
                    && url_followed_by_content
                    && !matches!(self.peek_at(self.pos + 1), Some('/' | '*')))
            {
                // Keep a URL on the next line inside the truncated string.
                if c == '\n' {
                    let mut next_start = self.pos + 1;
                    while self
                        .peek_at(next_start)
                        .is_some_and(|ch| matches!(ch, ' ' | '\t' | '\r'))
                    {
                        next_start += 1;
                    }
                    let mut slash_idx = next_start;
                    while self
                        .peek_at(slash_idx)
                        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == ':')
                    {
                        slash_idx += 1;
                    }
                    if self.looks_like_url_start(next_start, slash_idx)
                        || (self.peek_at(slash_idx) == Some('/')
                            && slash_idx + 1 == self.chars.len()
                            && self.ends_with_url_scheme(next_start, slash_idx))
                    {
                        self.parse_string_char(c)?;
                        continue;
                    }
                }

                // URL schemes can follow prose as well as start the string.
                if c == '/'
                    && self.pos > input_start + 1
                    && self.peek_at(self.pos - 1) == Some(':')
                    && self.ends_with_url_scheme(input_start + 1, self.pos)
                {
                    while self.peek().is_some_and(chars::is_url_char) {
                        let url_char = self.chars[self.pos];
                        if is_wrapper_close(&self.chars, self.pos, parenthesis_depth) {
                            break;
                        }
                        if url_char == '(' {
                            parenthesis_depth += 1;
                        } else if url_char == ')' {
                            parenthesis_depth = parenthesis_depth.saturating_sub(1);
                        }
                        self.output.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                    if self
                        .peek()
                        .is_some_and(|ch| !chars::is_unquoted_string_delimiter(ch))
                    {
                        // Later slashes belong to the continuing quoted content.
                        url_followed_by_content = true;
                        continue;
                    }
                }

                self.insert_before_last_output_whitespace(output_start + 1, "\"");
                self.parse_concatenated_string(is_wrapper_argument)?;
                return Ok(true);
            } else if c == '\\' {
                self.parse_string_escape()?;
            } else {
                if c == '(' {
                    parenthesis_depth += 1;
                } else if c == ')' {
                    parenthesis_depth = parenthesis_depth.saturating_sub(1);
                }
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
        self.pos += 1; // skip '\'
        let esc = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(());
            }
        };

        if matches!(esc, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') {
            self.output.push('\\');
            self.output.push(esc);
            self.pos += 1;
            return Ok(());
        }

        match esc {
            'u' => {
                let backslash_pos = self.pos.saturating_sub(1);
                let mut digits = 0;
                while digits < 4
                    && self
                        .peek_at(self.pos + 1 + digits)
                        .is_some_and(chars::is_hex)
                {
                    digits += 1;
                }

                if digits == 4 {
                    let code_unit = self.hex_quad(self.pos + 1).ok_or_else(|| {
                        self.error_at_kind(
                            "Invalid unicode escape",
                            backslash_pos,
                            JsonRepairErrorKind::InvalidUnicode,
                        )
                    })?;
                    if (0xD800..=0xDBFF).contains(&code_unit) {
                        let next = self.pos + 5;
                        let low = if self.peek_at(next) == Some('\\')
                            && self.peek_at(next + 1) == Some('u')
                        {
                            self.hex_quad(next + 2)
                        } else {
                            None
                        };
                        if !low.is_some_and(|unit| (0xDC00..=0xDFFF).contains(&unit)) {
                            return Err(self.error_at_kind(
                                "Invalid unicode surrogate pair",
                                backslash_pos,
                                JsonRepairErrorKind::InvalidUnicode,
                            ));
                        }
                        self.output.push_str("\\u");
                        for i in 0..4 {
                            self.output.push(self.chars[self.pos + 1 + i]);
                        }
                        self.output.push_str("\\u");
                        for i in 0..4 {
                            self.output.push(self.chars[next + 2 + i]);
                        }
                        self.pos += 11;
                        return Ok(());
                    }
                    if (0xDC00..=0xDFFF).contains(&code_unit) {
                        return Err(self.error_at_kind(
                            "Invalid unicode surrogate pair",
                            backslash_pos,
                            JsonRepairErrorKind::InvalidUnicode,
                        ));
                    }
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
                self.push_string_char(esc);
                self.pos += 1;
            }
        }
        Ok(())
    }

    fn hex_quad(&self, start: usize) -> Option<u16> {
        let mut value = 0u16;
        for offset in 0..4 {
            let digit = self.peek_at(start + offset)?.to_digit(16)? as u16;
            value = value * 16 + digit;
        }
        Some(value)
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

    fn parse_concatenated_string(&mut self, is_wrapper_argument: bool) -> Result<bool> {
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
            self.enter_container()?;
            let parsed = self.parse_string(is_wrapper_argument)?;
            self.leave_container();
            if parsed {
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
    pub(super) fn parse_unquoted_string(
        &mut self,
        is_key: bool,
        is_wrapper_argument: bool,
    ) -> Result<bool> {
        let start = self.pos;

        if !is_key
            && self.peek().is_some_and(chars::is_identifier_start)
            && self.maybe_known_wrapper_start()
            && self.parse_known_wrapper_call()?
        {
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
                    || (is_key && matches!(c, ':' | '='))
                    || (is_wrapper_argument && c == ')'))
            {
                break;
            }
            self.pos += 1;
        }

        if self.pos > start
            && self.peek_at(self.pos.saturating_sub(1)) == Some(':')
            && self.looks_like_url_start(start, self.pos)
        {
            while let Some(c) = self.peek().filter(|&c| chars::is_url_char(c)) {
                if is_wrapper_argument && c == ')' && parenthesis_depth == 0 {
                    break;
                }
                if c == '(' {
                    parenthesis_depth += 1;
                } else if c == ')' {
                    parenthesis_depth = parenthesis_depth.saturating_sub(1);
                }
                self.pos += 1;
            }
        }

        if self.pos == start {
            return Ok(false);
        }

        while self.pos > start && chars::is_whitespace(self.chars[self.pos - 1]) {
            self.pos -= 1;
        }

        // A whitespace-only token has no value. Form feed is trimmed here but
        // is not consumed by parse_whitespace_and_comments. Returning false
        // prevents the array parser from looping at an unchanged position.
        if self.pos == start {
            return Ok(false);
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

    fn ends_with_url_scheme(&self, start: usize, slash_idx: usize) -> bool {
        [
            "http:", "https:", "ftp:", "mailto:", "file:", "data:", "irc:",
        ]
        .iter()
        .any(|scheme| {
            slash_idx >= start + scheme.len()
                && self.slice_eq(slash_idx - scheme.len(), slash_idx, scheme)
        })
    }

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
    fn maybe_known_wrapper_start(&self) -> bool {
        match self.peek() {
            Some(c) => matches!(c, 'c' | 'C' | 'j' | 'J' | 'o' | 'O' | 'n' | 'N' | 'i' | 'I'),
            None => false,
        }
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
        let mut cursor = start + 1;
        let mut parentheses = 1usize;
        let mut containers = 0usize;
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
            if !in_url && (chars::is_quote(c) || chars::is_identifier_start(c)) {
                if chars::is_identifier_start(c) {
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
                } else {
                    self.parse_keyword_or_unquoted(containers == 0)
                };
                cursor = self.pos;
                self.pos = input_start;
                self.output.truncate(output_start);
                self.depth = depth_start;
                parsed?;
                continue;
            }
            if !in_url && c == '/' && self.peek_at(cursor + 1) == Some('*') {
                cursor += 2;
                while self.peek_at(cursor).is_some() && !self.matches_at(cursor, "*/") {
                    cursor += 1;
                }
                cursor += 2;
                continue;
            }
            if !in_url && ((c == '/' && self.peek_at(cursor + 1) == Some('/')) || c == '#') {
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
                '{' | '[' => containers += 1,
                '}' | ']' if containers == 0 => return Ok(false),
                '}' | ']' => containers -= 1,
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
            .parse_string(false)
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

    fn insert_before_last_output_whitespace(&mut self, start: usize, text: &str) {
        let bytes = self.output.as_bytes();
        let mut idx = bytes.len();
        while idx > start && matches!(bytes[idx - 1], b' ' | b'\n' | b'\r' | b'\t') {
            idx -= 1;
        }
        self.output.insert_str(idx, text);
    }
}
