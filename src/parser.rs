use crate::chars;
use crate::error::JsonRepairError;

type Result<T> = std::result::Result<T, JsonRepairError>;

/// Recursive-descent JSON repair parser.
///
/// Walks the input once, writing repaired output to an internal buffer.
/// Handles 30+ categories of broken JSON (quotes, commas, comments, keywords, etc.).
pub struct JsonRepairer {
    chars: Vec<char>,
    pos: usize,
    output: String,
}

impl JsonRepairer {
    pub fn new(input: &str) -> Self {
        let input = chars::strip_bom(input);
        Self {
            chars: input.chars().collect(),
            pos: 0,
            output: String::with_capacity(input.len()),
        }
    }

    pub fn repair(mut self) -> Result<String> {
        self.parse_whitespace_and_comments();

        let parsed = self.parse_value()?;
        if !parsed {
            // Empty or whitespace-only input
            if self.pos >= self.chars.len() {
                return Err(self.error("Unexpected end of input"));
            }
            return Err(self.error("Unexpected character"));
        }

        self.parse_whitespace_and_comments();

        // Handle NDJSON: multiple root values separated by newlines
        if self.pos < self.chars.len() && self.peek() != Some(')') {
            // Could be NDJSON or trailing content
            let has_newline =
                self.output.contains('\n') || self.chars[..self.pos].iter().any(|&c| c == '\n');

            if has_newline || self.peek().is_some() {
                // Try to parse as NDJSON (newline-delimited JSON)
                let mut values = vec![std::mem::take(&mut self.output)];
                self.output = String::new();

                while self.pos < self.chars.len() {
                    self.parse_whitespace_and_comments();
                    if self.pos >= self.chars.len() {
                        break;
                    }

                    if self.parse_value()? {
                        values.push(std::mem::take(&mut self.output));
                        self.output = String::new();
                        self.parse_whitespace_and_comments();
                    } else {
                        break;
                    }
                }

                if values.len() > 1 {
                    self.output = format!("[\n{}\n]", values.join(",\n"));
                } else {
                    self.output = values.into_iter().next().unwrap_or_default();
                }
            }
        }

        // Skip trailing JSONP closing paren
        if self.peek() == Some(')') {
            self.pos += 1;
            self.parse_whitespace_and_comments();
        }

        // Skip trailing semicolons
        while self.peek() == Some(';') {
            self.pos += 1;
            self.parse_whitespace_and_comments();
        }

        if self.pos < self.chars.len() {
            return Err(self.error("Unexpected trailing content"));
        }

        Ok(self.output)
    }

    // ── Value dispatch ──────────────────────────────────────

    fn parse_value(&mut self) -> Result<bool> {
        self.parse_whitespace_and_comments();

        let c = match self.peek() {
            Some(c) => c,
            None => return Ok(false),
        };

        // Object
        if c == '{' {
            return self.parse_object();
        }

        // Array
        if c == '[' {
            return self.parse_array();
        }

        // Markdown code fence (must check before quote, since backtick is a quote char)
        if c == '`' && self.matches_at(self.pos, "```") {
            return self.parse_markdown_fenced();
        }

        // String (any quote style)
        if chars::is_quote(c) {
            return self.parse_string();
        }

        // Number
        if chars::is_number_start(c) {
            return self.parse_number();
        }

        // Keywords (true, false, null, True, False, None, undefined, NaN, Infinity, etc.)
        if chars::is_identifier_start(c) {
            return self.parse_keyword_or_unquoted();
        }

        // JSONP function call: callback({...})
        if c == '(' {
            // Skip the opening paren and parse the inner value
            self.pos += 1;
            self.parse_whitespace_and_comments();
            return self.parse_value();
        }

        // Regex literal → treat as string
        if c == '/' {
            if self.matches_at(self.pos, "//") || self.matches_at(self.pos, "/*") {
                self.parse_whitespace_and_comments();
                return self.parse_value();
            }
            return self.parse_regex_as_string();
        }

        // Ellipsis (skip)
        if c == '.' && self.matches_at(self.pos, "...") {
            self.pos += 3;
            self.parse_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.pos += 1;
            }
            return Ok(false);
        }

        Ok(false)
    }

    // ── Object ──────────────────────────────────────────────

    fn parse_object(&mut self) -> Result<bool> {
        self.expect('{');
        self.output.push('{');
        self.parse_whitespace_and_comments();

        let mut first = true;

        loop {
            // Skip ellipsis in objects
            if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
                self.pos += 3;
                self.parse_whitespace_and_comments();
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                }
                continue;
            }

            // End of object
            if self.peek() == Some('}') {
                self.pos += 1;
                self.output.push('}');
                return Ok(true);
            }

            // End of input (truncated) — auto-close
            if self.pos >= self.chars.len() {
                self.output.push('}');
                return Ok(true);
            }

            // Comma handling
            if !first {
                // Expect a comma
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();

                    // Handle trailing comma before }
                    if self.peek() == Some('}') {
                        // Trailing comma — just skip it (don't output)
                        self.pos += 1;
                        self.output.push('}');
                        return Ok(true);
                    }

                    self.output.push(',');
                } else if self.peek() != Some('}') {
                    // Missing comma — insert one
                    self.output.push(',');
                }
            }
            first = false;

            // Skip leading commas
            while self.peek() == Some(',') {
                self.pos += 1;
                self.parse_whitespace_and_comments();
            }

            self.parse_whitespace_and_comments();

            // Ellipsis after comma (e.g. {"a": 1, ...})
            if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
                self.pos += 3;
                self.parse_whitespace_and_comments();
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                }
                // Remove trailing comma we already output
                if self.output.ends_with(',') {
                    self.output.pop();
                }
                continue;
            }

            // Parse key
            let key_parsed = if let Some(c) = self.peek() {
                if chars::is_quote(c) {
                    self.parse_string()?
                } else if c == '{' || c == '[' {
                    // Object/array as key is invalid, stop
                    false
                } else if c == '}' {
                    continue;
                } else {
                    // Unquoted key
                    self.parse_unquoted_key()?
                }
            } else {
                false
            };

            if !key_parsed {
                if self.pos >= self.chars.len() {
                    self.output.push('}');
                    return Ok(true);
                }
                return Err(self.error("Expected object key"));
            }

            self.parse_whitespace_and_comments();

            // Colon
            if self.peek() == Some(':') {
                self.pos += 1;
                self.output.push(':');
            } else if self.peek() == Some('=') {
                // Accept = as colon
                self.pos += 1;
                self.output.push(':');
            } else {
                // Missing colon — insert one
                self.output.push(':');
            }

            self.parse_whitespace_and_comments();

            // Parse value
            let value_parsed = self.parse_value()?;
            if !value_parsed {
                // Missing value — insert null
                self.output.push_str("null");
            }

            self.parse_whitespace_and_comments();
        }
    }

    // ── Array ───────────────────────────────────────────────

    fn parse_array(&mut self) -> Result<bool> {
        self.expect('[');
        self.output.push('[');
        self.parse_whitespace_and_comments();

        let mut first = true;

        loop {
            // Skip ellipsis
            if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
                self.pos += 3;
                self.parse_whitespace_and_comments();
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                }
                continue;
            }

            // End of array
            if self.peek() == Some(']') {
                self.pos += 1;
                self.output.push(']');
                return Ok(true);
            }

            // End of input (truncated) — auto-close
            if self.pos >= self.chars.len() {
                self.output.push(']');
                return Ok(true);
            }

            // Comma handling
            if !first {
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();

                    // Trailing comma before ]
                    if self.peek() == Some(']') {
                        self.pos += 1;
                        self.output.push(']');
                        return Ok(true);
                    }

                    self.output.push(',');
                } else if self.peek() != Some(']') {
                    // Missing comma
                    self.output.push(',');
                }
            }
            first = false;

            // Skip leading commas
            while self.peek() == Some(',') {
                self.pos += 1;
                self.parse_whitespace_and_comments();
            }

            // Ellipsis after comma (e.g. [1, 2, ...])
            if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
                self.pos += 3;
                self.parse_whitespace_and_comments();
                if self.peek() == Some(',') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                }
                if self.output.ends_with(',') {
                    self.output.pop();
                }
                continue;
            }

            // Parse value
            let parsed = self.parse_value()?;
            if !parsed {
                if self.pos >= self.chars.len() {
                    self.output.push(']');
                    return Ok(true);
                }
                // Unknown content — skip character
                self.pos += 1;
            }

            self.parse_whitespace_and_comments();
        }
    }

    // ── String ──────────────────────────────────────────────

    fn parse_string(&mut self) -> Result<bool> {
        let quote = match self.peek() {
            Some(c) if chars::is_quote(c) => c,
            _ => return Ok(false),
        };
        self.pos += 1;

        // Determine the matching closing quote
        let is_double = chars::is_double_quote_like(quote);
        let close_fn: fn(char) -> bool = if is_double {
            chars::is_double_quote_like
        } else {
            chars::is_single_quote_like
        };

        self.output.push('"');

        loop {
            match self.peek() {
                None => {
                    // Truncated string — auto-close
                    self.output.push('"');
                    return Ok(true);
                }
                Some(c) if close_fn(c) => {
                    self.pos += 1;
                    // Check for string concatenation: "a" + "b"
                    self.parse_whitespace_and_comments();
                    if self.peek() == Some('+') {
                        let save_pos = self.pos;
                        self.pos += 1;
                        self.parse_whitespace_and_comments();
                        if self.peek().is_some_and(|c| chars::is_quote(c)) {
                            // Concatenation — parse next string without closing/opening quotes
                            let next_quote = self.chars[self.pos];
                            self.pos += 1;
                            let next_is_double = chars::is_double_quote_like(next_quote);
                            let _next_close: fn(char) -> bool = if next_is_double {
                                chars::is_double_quote_like
                            } else {
                                chars::is_single_quote_like
                            };
                            continue;
                        } else {
                            self.pos = save_pos;
                        }
                    }
                    self.output.push('"');
                    return Ok(true);
                }
                Some('\\') => {
                    self.pos += 1;
                    match self.peek() {
                        None => {
                            // Truncated escape
                            self.output.push('"');
                            return Ok(true);
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
                                'u' => {
                                    self.output.push_str("\\u");
                                    // Read up to 4 hex digits
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
                                    // Pad with zeros if incomplete
                                    while count < 4 {
                                        self.output.push('0');
                                        count += 1;
                                    }
                                }
                                '\'' => {
                                    // \' → '  (not a valid JSON escape, just output the quote)
                                    self.output.push('\'');
                                }
                                '\n' | '\r' => {
                                    // Line continuation — skip
                                }
                                _ => {
                                    // Unknown escape — output character without backslash
                                    self.output.push(esc);
                                }
                            }
                        }
                    }
                }
                Some(c) => {
                    self.pos += 1;
                    // Escape special characters that need escaping in JSON strings
                    match c {
                        '"' => self.output.push_str("\\\""),
                        '\n' => self.output.push_str("\\n"),
                        '\r' => self.output.push_str("\\r"),
                        '\t' => self.output.push_str("\\t"),
                        '\x08' => self.output.push_str("\\b"),
                        '\x0C' => self.output.push_str("\\f"),
                        c if (c as u32) < 0x20 => {
                            // Control character — escape as \u00XX
                            self.output.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        _ => self.output.push(c),
                    }
                }
            }
        }
    }

    // ── Unquoted key ────────────────────────────────────────

    fn parse_unquoted_key(&mut self) -> Result<bool> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if chars::is_identifier_char(c) || c == '-' {
                self.pos += 1;
            } else {
                break;
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

    // ── Number ──────────────────────────────────────────────

    fn parse_number(&mut self) -> Result<bool> {
        let start = self.pos;

        // Optional minus
        if self.peek() == Some('-') {
            self.pos += 1;
        }

        // Integer part
        if self.peek() == Some('0') {
            self.pos += 1;
            // Check for leading zeros like 0789
            if self.peek().is_some_and(chars::is_digit) {
                // Leading zero followed by digits — parse as string
                while self.peek().is_some_and(chars::is_digit) {
                    self.pos += 1;
                }
                let num_str: String = self.chars[start..self.pos].iter().collect();
                self.output.push('"');
                self.output.push_str(&num_str);
                self.output.push('"');
                return Ok(true);
            }
        } else {
            // Non-zero integer
            if !self.peek().is_some_and(chars::is_digit) {
                if self.pos > start {
                    // Just a minus sign — not a number
                    self.pos = start;
                    return Ok(false);
                }
                return Ok(false);
            }
            while self.peek().is_some_and(chars::is_digit) {
                self.pos += 1;
            }
        }

        // Decimal part
        if self.peek() == Some('.') {
            self.pos += 1;
            if self.peek().is_some_and(chars::is_digit) {
                while self.peek().is_some_and(chars::is_digit) {
                    self.pos += 1;
                }
                // Check for multiple decimals: 0.0.1 → "0.0.1"
                if self.peek() == Some('.') {
                    while self.peek().is_some_and(|c| chars::is_digit(c) || c == '.') {
                        self.pos += 1;
                    }
                    let s: String = self.chars[start..self.pos].iter().collect();
                    self.output.push('"');
                    self.output.push_str(&s);
                    self.output.push('"');
                    return Ok(true);
                }
            } else {
                // Trailing dot: `2.` → `2.0`
                // Will be handled by output below — append 0
                let mut s: String = self.chars[start..self.pos].iter().collect();
                s.push('0');
                self.output.push_str(&s);
                return Ok(true);
            }
        }

        // Exponent part
        if self.peek().is_some_and(|c| c == 'e' || c == 'E') {
            self.pos += 1;
            if self.peek().is_some_and(|c| c == '+' || c == '-') {
                self.pos += 1;
            }
            if self.peek().is_some_and(chars::is_digit) {
                while self.peek().is_some_and(chars::is_digit) {
                    self.pos += 1;
                }
            } else {
                // Truncated exponent: `2e` → `2e0`
                let mut s: String = self.chars[start..self.pos].iter().collect();
                s.push('0');
                // Check if result is valid
                if self.peek() == Some('.') {
                    // 2e3.4 → "2e3.4" (as string)
                    while self.peek().is_some_and(|c| chars::is_digit(c) || c == '.') {
                        self.pos += 1;
                    }
                    let s: String = self.chars[start..self.pos].iter().collect();
                    self.output.push('"');
                    self.output.push_str(&s);
                    self.output.push('"');
                    return Ok(true);
                }
                self.output.push_str(&s);
                return Ok(true);
            }
        }

        if self.pos == start {
            return Ok(false);
        }

        let num_str: String = self.chars[start..self.pos].iter().collect();
        self.output.push_str(&num_str);
        Ok(true)
    }

    // ── Keywords and unquoted strings ───────────────────────

    fn parse_keyword_or_unquoted(&mut self) -> Result<bool> {
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
            // Standard JSON keywords
            "true" | "false" | "null" => {
                self.output.push_str(&word);
            }
            // Python keywords
            "True" => self.output.push_str("true"),
            "False" => self.output.push_str("false"),
            "None" => self.output.push_str("null"),
            // JavaScript keywords
            "undefined" => self.output.push_str("null"),
            "NaN" => self.output.push_str("null"),
            "Infinity" => self.output.push_str("null"),
            // MongoDB constructors: ObjectId("..."), NumberLong("...")
            "ObjectId" | "NumberLong" | "NumberInt" | "NumberDecimal" | "ISODate" => {
                self.parse_whitespace_and_comments();
                if self.peek() == Some('(') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                    let parsed = self.parse_value()?;
                    if !parsed {
                        self.output.push_str("null");
                    }
                    self.parse_whitespace_and_comments();
                    if self.peek() == Some(')') {
                        self.pos += 1;
                    }
                } else {
                    // Just the word — treat as unquoted string
                    self.output.push('"');
                    self.output.push_str(&word);
                    self.output.push('"');
                }
            }
            _ => {
                // Check if followed by `(` — function call like callback(...)
                self.parse_whitespace_and_comments();
                if self.peek() == Some('(') {
                    self.pos += 1;
                    self.parse_whitespace_and_comments();
                    let parsed = self.parse_value()?;
                    if !parsed {
                        self.output.push_str("null");
                    }
                    self.parse_whitespace_and_comments();
                    if self.peek() == Some(')') {
                        self.pos += 1;
                    }
                    return Ok(true);
                }
                // Unquoted string — wrap in quotes
                self.output.push('"');
                self.output.push_str(&word);
                self.output.push('"');
            }
        }

        Ok(true)
    }

    // ── Markdown fenced code block ──────────────────────────

    fn parse_markdown_fenced(&mut self) -> Result<bool> {
        // Skip opening ```
        self.pos += 3;
        // Skip optional language tag (e.g., `json`)
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                self.pos += 1;
                break;
            }
            self.pos += 1;
        }

        // Find closing ``` and collect content between
        let content_start = self.pos;
        let mut content_end = self.chars.len();
        while self.pos < self.chars.len() {
            if self.matches_at(self.pos, "```") {
                content_end = self.pos;
                self.pos += 3;
                // Skip rest of line after closing ```
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        self.pos += 1;
                        break;
                    }
                    self.pos += 1;
                }
                break;
            }
            self.pos += 1;
        }

        let content: String = self.chars[content_start..content_end].iter().collect();
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Ok(false);
        }

        // Re-parse the fenced content
        let inner = JsonRepairer::new(trimmed);
        match inner.repair() {
            Ok(repaired) => {
                self.output.push_str(&repaired);
                Ok(true)
            }
            Err(_) => {
                // If inner parse fails, treat as string
                self.output.push('"');
                for c in trimmed.chars() {
                    match c {
                        '"' => self.output.push_str("\\\""),
                        '\\' => self.output.push_str("\\\\"),
                        '\n' => self.output.push_str("\\n"),
                        '\r' => self.output.push_str("\\r"),
                        '\t' => self.output.push_str("\\t"),
                        _ => self.output.push(c),
                    }
                }
                self.output.push('"');
                Ok(true)
            }
        }
    }

    // ── Regex literal → string ──────────────────────────────

    fn parse_regex_as_string(&mut self) -> Result<bool> {
        if self.peek() != Some('/') {
            return Ok(false);
        }
        self.pos += 1;
        self.output.push('"');
        self.output.push('/');

        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => {
                    self.output.push('/');
                    self.output.push('"');
                    return Ok(true);
                }
                Some('/') => {
                    self.pos += 1;
                    self.output.push('/');
                    // Skip flags
                    while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                        self.output.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                    self.output.push('"');
                    return Ok(true);
                }
                Some('\\') => {
                    self.pos += 1;
                    self.output.push_str("\\\\");
                    if let Some(c) = self.peek() {
                        self.pos += 1;
                        if c == '"' {
                            self.output.push_str("\\\"");
                        } else {
                            self.output.push(c);
                        }
                    }
                }
                Some(c) => {
                    self.pos += 1;
                    if c == '"' {
                        self.output.push_str("\\\"");
                    } else {
                        self.output.push(c);
                    }
                }
            }
        }
    }

    // ── Whitespace and comments ─────────────────────────────

    fn parse_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.peek().is_some_and(chars::is_whitespace) {
                self.pos += 1;
            }

            // Line comment //
            if self.matches_at(self.pos, "//") {
                self.pos += 2;
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        self.pos += 1;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            // Block comment /* ... */
            if self.matches_at(self.pos, "/*") {
                self.pos += 2;
                while self.pos < self.chars.len() {
                    if self.matches_at(self.pos, "*/") {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            // Hash comment #
            if self.peek() == Some('#') {
                self.pos += 1;
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        self.pos += 1;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            break;
        }
    }

    // ── Helpers ─────────────────────────────────────────────

    #[inline]
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn expect(&mut self, expected: char) {
        debug_assert_eq!(self.peek(), Some(expected));
        self.pos += 1;
    }

    /// Check if the input matches `pattern` at the given position.
    fn matches_at(&self, pos: usize, pattern: &str) -> bool {
        let pat_chars: Vec<char> = pattern.chars().collect();
        if pos + pat_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &pc) in pat_chars.iter().enumerate() {
            if self.chars[pos + i] != pc {
                return false;
            }
        }
        true
    }

    fn error(&self, message: &str) -> JsonRepairError {
        JsonRepairError::new(message, self.pos)
    }
}
