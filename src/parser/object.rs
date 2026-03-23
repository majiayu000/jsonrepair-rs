use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_object(&mut self) -> Result<bool> {
        self.output.push('{');
        self.pos += 1;
        self.parse_whitespace_and_comments();

        if self.parse_skip_ellipsis() {
            // object starts with ellipsis
        }

        let mut initial = true;
        while !self.at_end() && self.peek() != Some('}') {
            if !initial {
                let has_comma = self.parse_char(',');
                self.parse_whitespace_and_comments();
                self.parse_skip_ellipsis();

                if !has_comma && self.peek() != Some('}') && !self.at_end() {
                    // Missing comma — insert before trailing whitespace
                    self.insert_before_last_whitespace(",");
                }
            }

            // Skip leading commas
            while self.peek() == Some(',') {
                self.pos += 1;
                self.parse_whitespace_and_comments();
            }

            // Trailing comma → end of object
            let is_end = self.peek() == Some('}') || self.at_end();
            if is_end {
                self.strip_last_occurrence(',');
                break;
            }

            // Parse key
            let key_parsed = self.parse_object_key()?;
            if !key_parsed {
                if self.at_end() {
                    break;
                }
                // Mismatched bracket — treat as end
                if self.peek() == Some(']') {
                    self.pos += 1;
                    break;
                }
                return Err(self.error("Object key expected"));
            }

            self.parse_whitespace_and_comments();

            // Colon
            if self.peek() == Some(':') {
                self.output.push(':');
                self.pos += 1;
            } else if self.peek() == Some('=') {
                self.output.push(':');
                self.pos += 1;
            } else if self.peek() != Some('{') && self.peek() != Some('[') {
                // Missing colon — check if it looks like colon was omitted
                self.insert_before_last_whitespace(":");
            }

            self.parse_whitespace_and_comments();

            // Parse value
            if !self.parse_value()? {
                // Missing value → null
                self.output.push_str("null");
            }

            self.parse_whitespace_and_comments();
            initial = false;
        }

        if self.peek() == Some('}') {
            self.output.push('}');
            self.pos += 1;
        } else {
            // Truncated — auto-close
            self.strip_last_occurrence(',');
            self.output.push('}');
        }
        Ok(true)
    }

    fn parse_object_key(&mut self) -> Result<bool> {
        if let Some(c) = self.peek() {
            if chars::is_quote(c) {
                return self.parse_string(false);
            }
            if c == '{' || c == '[' {
                return Ok(false);
            }
            if c == '}' {
                return Ok(false);
            }
            // Unquoted key (identifier, number, etc.)
            return self.parse_unquoted_key();
        }
        Ok(false)
    }

    /// Parse and skip `...` (ellipsis), returning true if found.
    pub(super) fn parse_skip_ellipsis(&mut self) -> bool {
        if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
            self.pos += 3;
            self.parse_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.pos += 1;
                self.parse_whitespace_and_comments();
            }
            self.parse_skip_ellipsis(); // nested ellipsis
            true
        } else {
            false
        }
    }
}
