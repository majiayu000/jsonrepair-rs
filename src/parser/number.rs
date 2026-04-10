use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    #[inline(always)]
    pub(super) fn parse_number(&mut self) -> Result<bool> {
        let start = self.pos;
        let mut append_trailing_zero = false;

        if self.peek() == Some('-') {
            self.pos += 1;
            if self.at_end_of_number() {
                append_trailing_zero = true;
            } else if !self.peek().is_some_and(chars::is_digit) {
                self.pos = start;
                return Ok(false);
            }
        }

        if !append_trailing_zero {
            while self.peek().is_some_and(chars::is_digit) {
                self.pos += 1;
            }

            if self.peek() == Some('.') {
                self.pos += 1;
                if self.at_end_of_number() {
                    append_trailing_zero = true;
                } else if !self.peek().is_some_and(chars::is_digit) {
                    self.pos = start;
                    return Ok(false);
                } else {
                    while self.peek().is_some_and(chars::is_digit) {
                        self.pos += 1;
                    }
                }
            }

            if !append_trailing_zero && self.peek().is_some_and(|c| c == 'e' || c == 'E') {
                self.pos += 1;
                if self.peek().is_some_and(|c| c == '-' || c == '+') {
                    self.pos += 1;
                }
                if self.at_end_of_number() {
                    append_trailing_zero = true;
                } else if !self.peek().is_some_and(chars::is_digit) {
                    self.pos = start;
                    return Ok(false);
                } else {
                    while self.peek().is_some_and(chars::is_digit) {
                        self.pos += 1;
                    }
                }
            }
        }

        // If number is followed by non-delimiter text, let unquoted string parser handle it.
        if !append_trailing_zero && !self.at_end_of_number() {
            self.pos = start;
            return Ok(false);
        }

        if self.pos > start {
            self.push_number_to_output(start, self.pos, append_trailing_zero);
            return Ok(true);
        }

        Ok(false)
    }

    #[inline(always)]
    fn push_number_to_output(&mut self, start: usize, end: usize, append_trailing_zero: bool) {
        if self.has_invalid_leading_zero(start, end) {
            self.output.push('"');
            self.push_slice_to_output(start, end);
            if append_trailing_zero {
                self.output.push('0');
            }
            self.output.push('"');
            return;
        }

        self.push_slice_to_output(start, end);
        if append_trailing_zero {
            self.output.push('0');
        }
    }

    #[inline(always)]
    fn has_invalid_leading_zero(&self, start: usize, end: usize) -> bool {
        let integer_start = if self.peek_at(start) == Some('-') {
            start + 1
        } else {
            start
        };

        if integer_start >= end || !self.chars[integer_start].is_ascii_digit() {
            return false;
        }

        let mut integer_end = integer_start;
        while integer_end < end && self.chars[integer_end].is_ascii_digit() {
            integer_end += 1;
        }

        integer_end > integer_start + 1 && self.chars[integer_start] == '0'
    }

    #[inline(always)]
    fn at_end_of_number(&self) -> bool {
        match self.peek() {
            None => true,
            Some(c) => chars::is_delimiter(c) || chars::is_whitespace(c),
        }
    }
}
