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
            let num: String = self.chars[start..self.pos].iter().collect();
            let has_invalid_leading_zero =
                num.starts_with('0') && num.chars().nth(1).is_some_and(|c| c.is_ascii_digit());
            if has_invalid_leading_zero {
                self.output.push('"');
                self.output.push_str(&num);
                self.output.push('"');
            } else {
                self.output.push_str(&num);
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

    fn repair_number_ending_with_numeric_symbol(&mut self, start: usize) {
        let mut s: String = self.chars[start..self.pos].iter().collect();
        s.push('0');
        self.output.push_str(&s);
    }
}
