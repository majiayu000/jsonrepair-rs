use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_array(&mut self) -> Result<bool> {
        if self.peek() != Some('[') {
            return Ok(false);
        }

        self.output.push('[');
        self.pos += 1;
        self.parse_whitespace_and_comments();

        // Skip leading comma: [,1,2]
        if self.skip_char(',') {
            self.parse_whitespace_and_comments();
        }

        let mut initial = true;
        while !self.at_end() && self.peek() != Some(']') {
            if !initial {
                let processed_comma = self.parse_char(',');
                if !processed_comma {
                    // Missing comma.
                    self.insert_before_last_whitespace(",");
                }
            } else {
                initial = false;
            }

            self.parse_skip_ellipsis();

            let processed_value = self.parse_value()?;
            if !processed_value {
                // Trailing comma or truncated input.
                self.strip_last_occurrence(',');
                break;
            }
        }

        if self.peek() == Some(']') {
            self.output.push(']');
            self.pos += 1;
        } else {
            // Missing closing array bracket.
            self.insert_before_last_whitespace("]");
        }
        Ok(true)
    }
}
