use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_array(&mut self) -> Result<bool> {
        self.output.push('[');
        self.pos += 1;
        self.parse_whitespace_and_comments();

        if self.parse_skip_ellipsis() {
            // array starts with ellipsis
        }

        let mut initial = true;
        while !self.at_end() && self.peek() != Some(']') {
            if !initial {
                let has_comma = self.parse_char(',');
                self.parse_whitespace_and_comments();
                self.parse_skip_ellipsis();

                if !has_comma && self.peek() != Some(']') && !self.at_end() {
                    // Missing comma
                    self.insert_before_last_whitespace(",");
                }
            }

            // Skip leading commas
            while self.peek() == Some(',') {
                self.pos += 1;
                self.parse_whitespace_and_comments();
            }

            // Trailing comma → end
            let is_end = self.peek() == Some(']') || self.at_end();
            if is_end {
                self.strip_last_occurrence(',');
                break;
            }

            // Parse value
            let parsed = self.parse_value()?;
            if !parsed {
                if self.at_end() {
                    break;
                }
                // Mismatched bracket — treat as end
                if self.peek() == Some('}') {
                    self.pos += 1;
                    break;
                }
                // Unknown content — skip
                self.pos += 1;
            }

            self.parse_whitespace_and_comments();
            initial = false;
        }

        if self.peek() == Some(']') {
            self.output.push(']');
            self.pos += 1;
        } else {
            // Truncated — auto-close
            self.strip_last_occurrence(',');
            self.output.push(']');
        }
        Ok(true)
    }
}
