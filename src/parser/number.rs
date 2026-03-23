use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_number(&mut self) -> Result<bool> {
        let start = self.pos;

        // Optional minus
        if self.peek() == Some('-') {
            self.pos += 1;
        }

        // Integer part
        if self.peek() == Some('0') {
            self.pos += 1;
            if self.peek().is_some_and(chars::is_digit) {
                // Leading zeros: 0789 → "0789"
                return self.parse_leading_zero_number(start);
            }
        } else if self.peek().is_some_and(chars::is_digit) {
            while self.peek().is_some_and(chars::is_digit) {
                self.pos += 1;
            }
        } else {
            // Just a minus, or nothing — handle truncated minus
            if self.pos > start {
                // bare `-` → `-0`
                let s: String = self.chars[start..self.pos].iter().collect();
                self.output.push_str(&s);
                self.output.push('0');
                return Ok(true);
            }
            return Ok(false);
        }

        // Decimal part
        if self.peek() == Some('.') {
            self.pos += 1;
            if self.peek().is_some_and(chars::is_digit) {
                while self.peek().is_some_and(chars::is_digit) {
                    self.pos += 1;
                }
                if self.peek() == Some('.') {
                    return self.parse_multi_decimal(start);
                }
            } else {
                // Trailing dot: 2. → 2.0
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
                // Truncated exponent: 2e → 2e0
                if self.peek() == Some('.') {
                    return self.parse_invalid_exp_decimal(start);
                }
                let mut s: String = self.chars[start..self.pos].iter().collect();
                s.push('0');
                self.output.push_str(&s);
                return Ok(true);
            }
        }

        if self.pos == start {
            return Ok(false);
        }

        let s: String = self.chars[start..self.pos].iter().collect();
        self.output.push_str(&s);
        Ok(true)
    }

    fn parse_leading_zero_number(&mut self, start: usize) -> Result<bool> {
        while self
            .peek()
            .is_some_and(|c| chars::is_digit(c) || c == '.' || c == 'e' || c == 'E')
        {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.output.push('"');
        self.output.push_str(&s);
        self.output.push('"');
        Ok(true)
    }

    fn parse_multi_decimal(&mut self, start: usize) -> Result<bool> {
        while self.peek().is_some_and(|c| chars::is_digit(c) || c == '.') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.output.push('"');
        self.output.push_str(&s);
        self.output.push('"');
        Ok(true)
    }

    fn parse_invalid_exp_decimal(&mut self, start: usize) -> Result<bool> {
        while self.peek().is_some_and(|c| chars::is_digit(c) || c == '.') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.output.push('"');
        self.output.push_str(&s);
        self.output.push('"');
        Ok(true)
    }
}
