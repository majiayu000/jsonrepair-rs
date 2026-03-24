use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_object(&mut self) -> Result<bool> {
        if self.peek() != Some('{') {
            return Ok(false);
        }

        self.output.push('{');
        self.pos += 1;
        self.parse_whitespace_and_comments();

        // Skip leading comma: {, "a": 1}
        if self.skip_char(',') {
            self.parse_whitespace_and_comments();
        }

        let mut initial = true;
        while !self.at_end() && self.peek() != Some('}') {
            if !initial {
                let processed_comma = self.parse_char(',');
                if !processed_comma {
                    // Missing comma.
                    self.insert_before_last_whitespace(",");
                }
                self.parse_whitespace_and_comments();
            } else {
                initial = false;
            }

            self.parse_skip_ellipsis();

            let processed_key = self.parse_object_key()?;
            if !processed_key {
                let near_end = self.at_end()
                    || matches!(self.peek(), Some('}') | Some('{') | Some(']') | Some('['));
                if near_end {
                    // Trailing comma.
                    self.strip_last_occurrence(',');
                } else {
                    return Err(self.error("Object key expected"));
                }
                break;
            }

            self.parse_whitespace_and_comments();
            let processed_colon = if self.parse_char(':') {
                true
            } else if self.peek() == Some('=') {
                // Non-standard, commonly seen in JS-ish objects.
                self.output.push(':');
                self.pos += 1;
                true
            } else {
                false
            };

            let truncated = self.at_end();
            if !processed_colon {
                if self.peek().is_some_and(chars::is_start_of_value) || truncated {
                    // Missing colon.
                    self.insert_before_last_whitespace(":");
                } else {
                    return Err(self.error("Colon expected"));
                }
            }

            let processed_value = self.parse_value()?;
            if !processed_value {
                if processed_colon || truncated {
                    // Missing object value.
                    self.output.push_str("null");
                } else {
                    return Err(self.error("Colon expected"));
                }
            }
        }

        if self.peek() == Some('}') {
            self.output.push('}');
            self.pos += 1;
        } else {
            // Missing closing brace.
            self.insert_before_last_whitespace("}");
        }

        Ok(true)
    }

    fn parse_object_key(&mut self) -> Result<bool> {
        if self.parse_string(false)? {
            return Ok(true);
        }
        self.parse_unquoted_string(true)
    }

    /// Parse and skip `...` (ellipsis), returning true if found.
    pub(super) fn parse_skip_ellipsis(&mut self) -> bool {
        self.parse_whitespace_and_comments();
        if self.peek() == Some('.') && self.matches_at(self.pos, "...") {
            self.pos += 3;
            self.parse_whitespace_and_comments();
            if self.peek() == Some(',') {
                self.pos += 1;
            }
            self.parse_skip_ellipsis();
            true
        } else {
            false
        }
    }
}
