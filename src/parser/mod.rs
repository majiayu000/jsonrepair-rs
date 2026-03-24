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
        if self.peek() == Some('{') {
            let processed = self.parse_object()?;
            self.parse_whitespace_and_comments();
            return Ok(processed);
        }
        if self.peek() == Some('[') {
            let processed = self.parse_array()?;
            self.parse_whitespace_and_comments();
            return Ok(processed);
        }
        if self.peek() == Some('`') && self.matches_at(self.pos, "```") {
            let processed = self.parse_markdown_fenced()?;
            self.parse_whitespace_and_comments();
            return Ok(processed);
        }
        if self.peek().is_some_and(chars::is_quote)
            || (self.peek() == Some('\\')
                && self.peek_at(self.pos + 1).is_some_and(chars::is_quote))
        {
            let processed = self.parse_string(false)?;
            self.parse_whitespace_and_comments();
            return Ok(processed);
        }
        if self.peek().is_some_and(chars::is_number_start) && self.parse_number()? {
            self.parse_whitespace_and_comments();
            return Ok(true);
        }
        if self.peek().is_some_and(chars::is_identifier_start)
            && self.parse_keyword_or_unquoted()?
        {
            self.parse_whitespace_and_comments();
            return Ok(true);
        }
        if self.parse_unquoted_string(false)? {
            self.parse_whitespace_and_comments();
            return Ok(true);
        }
        if self.peek() == Some('/') {
            let processed = self.parse_slash()?;
            self.parse_whitespace_and_comments();
            return Ok(processed);
        }

        self.parse_whitespace_and_comments();
        Ok(false)
    }

    fn parse_slash(&mut self) -> Result<bool> {
        self.parse_regex_as_string()
    }

    // ── Whitespace / comments ───────────────────────────────

    /// Copy whitespace to output, strip comments. Returns true if anything was consumed.
    pub(super) fn parse_whitespace_and_comments(&mut self) -> bool {
        self.parse_whitespace_and_comments_with_newline(true)
    }

    /// Copy whitespace to output, strip comments.
    /// When `skip_newline` is false, newlines are not consumed as whitespace.
    pub(super) fn parse_whitespace_and_comments_with_newline(
        &mut self,
        skip_newline: bool,
    ) -> bool {
        let start = self.pos;
        loop {
            while let Some(c) = self.peek() {
                let is_ws = if skip_newline {
                    chars::is_whitespace(c)
                } else {
                    chars::is_whitespace_except_newline(c)
                };
                if is_ws {
                    if chars::is_special_whitespace(c) {
                        self.output.push(' ');
                    } else {
                        self.output.push(c);
                    }
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

    #[inline]
    pub(super) fn peek_at(&self, idx: usize) -> Option<char> {
        self.chars.get(idx).copied()
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

    /// If next char equals `c`, advance without copying to output.
    pub(super) fn skip_char(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
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

    /// Find previous non-whitespace character index from `start` backwards.
    pub(super) fn prev_non_whitespace_index(&self, start: usize) -> Option<usize> {
        let mut idx = start;
        loop {
            let c = self.peek_at(idx)?;
            if !chars::is_whitespace(c) {
                return Some(idx);
            }
            if idx == 0 {
                return None;
            }
            idx -= 1;
        }
    }

    /// True when output ends with comma or newline followed by optional spaces/tabs/cr.
    pub(super) fn output_ends_with_comma_or_newline(&self) -> bool {
        let mut it = self.output.chars().rev();
        loop {
            match it.next() {
                Some(' ') | Some('\t') | Some('\r') => continue,
                Some(',') | Some('\n') => return true,
                _ => return false,
            }
        }
    }
}
