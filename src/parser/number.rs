use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    #[inline(always)]
    pub(super) fn parse_number(&mut self) -> Result<bool> {
        let start = self.pos;
        let mut append_trailing_zero = false;
        let mut has_leading_dot = false;
        let mut has_invalid_leading_zero = false;

        if self.peek() == Some('-') {
            self.pos += 1;
            if self.at_end_of_number() {
                append_trailing_zero = true;
            } else if !(self.peek().is_some_and(chars::is_digit)
                || (self.peek() == Some('.')
                    && self.peek_at(self.pos + 1).is_some_and(chars::is_digit)))
            {
                self.pos = start;
                return Ok(false);
            }
        }

        if !append_trailing_zero {
            if self.peek() == Some('.') {
                has_leading_dot = true;
                self.pos += 1;

                while self.peek().is_some_and(chars::is_digit) {
                    self.pos += 1;
                }
            } else {
                let mut integer_digits = 0usize;
                let mut first_integer_digit = '\0';
                while let Some(c) = self.peek() {
                    if chars::is_digit(c) {
                        if integer_digits == 0 {
                            first_integer_digit = c;
                        } else if integer_digits == 1 && first_integer_digit == '0' {
                            has_invalid_leading_zero = true;
                        }
                        integer_digits += 1;
                        self.pos += 1;
                    } else {
                        break;
                    }
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
            self.push_number_to_output(
                start,
                self.pos,
                append_trailing_zero,
                has_invalid_leading_zero,
                has_leading_dot,
            );
            return Ok(true);
        }

        Ok(false)
    }

    #[inline(always)]
    fn push_number_to_output(
        &mut self,
        start: usize,
        end: usize,
        append_trailing_zero: bool,
        has_invalid_leading_zero: bool,
        has_leading_dot: bool,
    ) {
        if has_invalid_leading_zero {
            self.output.push('"');
            self.push_slice_to_output(start, end);
            if append_trailing_zero {
                self.output.push('0');
            }
            self.output.push('"');
            return;
        }

        if has_leading_dot {
            if self.peek_at(start) == Some('-') {
                self.output.push('-');
                self.output.push('0');
                self.output.push('.');
                self.push_slice_to_output(start + 2, end);
            } else {
                self.output.push('0');
                self.output.push('.');
                self.push_slice_to_output(start + 1, end);
            }
        } else {
            self.push_slice_to_output(start, end);
        }
        if append_trailing_zero {
            self.output.push('0');
        }
    }

    #[inline(always)]
    fn at_end_of_number(&self) -> bool {
        match self.peek() {
            None => true,
            Some(c) => chars::is_delimiter(c) || chars::is_whitespace(c),
        }
    }
}
