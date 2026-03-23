use crate::chars;
use crate::error::JsonRepairError;

pub(crate) type Result<T> = std::result::Result<T, JsonRepairError>;

mod array;
mod format;
mod keyword;
mod number;
mod object;
mod string;
mod toplevel;

/// Recursive-descent JSON repair parser.
/// Copy-on-repair: preserves original whitespace, only modifies what needs fixing.
pub struct JsonRepairer {
    pub(super) chars: Vec<char>,
    pub(super) pos: usize,
    pub(super) output: String,
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

    // repair() and parse_ndjson() are in toplevel.rs
    pub(super) fn parse_value(&mut self) -> Result<bool> {
        self.parse_whitespace_and_comments();
        let c = match self.peek() {
            Some(c) => c,
            None => return Ok(false),
        };

        if c == '{' {
            return self.parse_object();
        }
        if c == '[' {
            return self.parse_array();
        }
        if c == '`' && self.matches_at(self.pos, "```") {
            return self.parse_markdown_fenced();
        }
        if chars::is_quote(c) {
            return self.parse_string(false);
        }
        if chars::is_number_start(c) {
            return self.parse_number();
        }
        if chars::is_identifier_start(c) {
            return self.parse_keyword_or_unquoted();
        }
        if c == '(' {
            self.pos += 1;
            return self.parse_value();
        }
        if c == '/' {
            return self.parse_slash();
        }
        Ok(false)
    }

    fn parse_slash(&mut self) -> Result<bool> {
        if self.matches_at(self.pos, "//") || self.matches_at(self.pos, "/*") {
            self.parse_whitespace_and_comments();
            return self.parse_value();
        }
        self.parse_regex_as_string()
    }

    // ── Whitespace / comments ───────────────────────────────

    /// Copy whitespace to output, strip comments. Returns true if anything was consumed.
    pub(super) fn parse_whitespace_and_comments(&mut self) -> bool {
        let start = self.pos;
        loop {
            while let Some(c) = self.peek() {
                if chars::is_whitespace(c) {
                    self.output.push(c);
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.matches_at(self.pos, "//") {
                self.pos += 2;
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            if self.matches_at(self.pos, "/*") {
                self.pos += 2;
                while self.pos < self.chars.len() && !self.matches_at(self.pos, "*/") {
                    self.pos += 1;
                }
                if self.matches_at(self.pos, "*/") {
                    self.pos += 2;
                }
                continue;
            }
            if self.peek() == Some('#') {
                self.pos += 1;
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
        self.pos > start
    }

    // ── Helpers ─────────────────────────────────────────────

    #[inline]
    pub(super) fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    #[inline]
    pub(super) fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// If next char equals `c`, copy it to output and advance. Returns true if matched.
    pub(super) fn parse_char(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.output.push(c);
            self.pos += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn matches_at(&self, pos: usize, pattern: &str) -> bool {
        let pat: Vec<char> = pattern.chars().collect();
        if pos + pat.len() > self.chars.len() {
            return false;
        }
        pat.iter()
            .enumerate()
            .all(|(i, &pc)| self.chars[pos + i] == pc)
    }

    /// Remove last occurrence of `c` from output.
    pub(super) fn strip_last_occurrence(&mut self, c: char) {
        if let Some(idx) = self.output.rfind(c) {
            self.output.remove(idx);
        }
    }

    fn strip_last_occurrence_in(&self, s: &mut String, c: char) {
        if let Some(idx) = s.rfind(c) {
            s.remove(idx);
        }
    }

    /// Insert `text` before any trailing whitespace in the output buffer.
    pub(super) fn insert_before_last_whitespace(&mut self, text: &str) {
        let bytes = self.output.as_bytes();
        let mut idx = bytes.len();
        while idx > 0 && matches!(bytes[idx - 1], b' ' | b'\n' | b'\r' | b'\t') {
            idx -= 1;
        }
        self.output.insert_str(idx, text);
    }

    pub(super) fn error(&self, msg: &str) -> JsonRepairError {
        JsonRepairError::new(msg, self.pos)
    }

    pub(super) fn error_char(&self, prefix: &str) -> JsonRepairError {
        if let Some(c) = self.peek() {
            JsonRepairError::new(format!("{prefix} \"{c}\""), self.pos)
        } else {
            JsonRepairError::new(prefix, self.pos)
        }
    }
}
