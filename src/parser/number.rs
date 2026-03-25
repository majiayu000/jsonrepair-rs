use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_number(&mut self) -> Result<bool> {
        let start = self.pos;

        if self.peek() == Some('-') {
            self.pos += 1;
            if self.at_end_of_number() {
                self.repair_number_ending_with_numeric_symbol(start);
                return Ok(true);
            }
            if !self.peek().is_some_and(chars::is_digit) {
                self.pos = start;
                return Ok(false);
            }
        }

        while self.peek().is_some_and(chars::is_digit) {
            self.pos += 1;
        }

        if self.peek() == Some('.') {
            self.pos += 1;
            if self.at_end_of_number() {
                self.repair_number_ending_with_numeric_symbol(start);
                return Ok(true);
            }
            if !self.peek().is_some_and(chars::is_digit) {
                self.pos = start;
                return Ok(false);
            }
            while self.peek().is_some_and(chars::is_digit) {
                self.pos += 1;
            }
        }

        if self.peek().is_some_and(|c| c == 'e' || c == 'E') {
            self.pos += 1;
            if self.peek().is_some_and(|c| c == '-' || c == '+') {
                self.pos += 1;
            }
            if self.at_end_of_number() {
                self.repair_number_ending_with_numeric_symbol(start);
                return Ok(true);
            }
            if !self.peek().is_some_and(chars::is_digit) {
                self.pos = start;
                return Ok(false);
            }
            while self.peek().is_some_and(chars::is_digit) {
                self.pos += 1;
            }
        }

        // If number is followed by non-delimiter text, let unquoted string parser handle it.
        if !self.at_end_of_number() {
            self.pos = start;
            return Ok(false);
        }

        if self.pos > start {
            // Check for leading zeros directly on char slice — no String allocation.
            let has_invalid_leading_zero = self.chars[start] == '0'
                && self.pos > start + 1
                && self.chars[start + 1].is_ascii_digit();
            if has_invalid_leading_zero {
                self.output.push('"');
                self.push_slice_to_output(start, self.pos);
                self.output.push('"');
            } else {
                self.push_slice_to_output(start, self.pos);
            }
            return Ok(true);
        }

        Ok(false)
    }

    fn at_end_of_number(&self) -> bool {
        match self.peek() {
            None => true,
            Some(c) => chars::is_delimiter(c) || chars::is_whitespace(c),
        }
    }

    /// Repair a truncated number (e.g. "-" → "-0", "3." → "3.0").
    /// Writes directly to output — no intermediate String.
    fn repair_number_ending_with_numeric_symbol(&mut self, start: usize) {
        self.push_slice_to_output(start, self.pos);
        self.output.push('0');
    }
}
