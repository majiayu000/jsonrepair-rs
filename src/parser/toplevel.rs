use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub fn repair(mut self) -> Result<String> {
        let processed = self.parse_value()?;
        let processed_comma = self.parse_char(',');
        self.parse_whitespace_and_comments();

        if processed_comma {
            self.strip_last_occurrence(',');
        }

        if self.is_jsonp_close() {
            self.pos += 1; // skip )
            self.parse_whitespace_and_comments();
        }

        if processed {
            self.skip_semicolons();
            self.parse_whitespace_and_comments();

            if self.at_end() {
                return Ok(self.output);
            }

            // NDJSON or comma-separated values
            return self.parse_ndjson();
        }

        if self.at_end() {
            return Err(self.error("Unexpected end of json string"));
        }
        Err(self.error_char("Unexpected character"))
    }

    fn parse_ndjson(mut self) -> Result<String> {
        let mut first_value = std::mem::take(&mut self.output);
        let mut values = Vec::new();

        let has_comma = first_value.ends_with(',');
        if has_comma {
            self.strip_last_occurrence_in(&mut first_value, ',');
        }
        values.push(first_value);

        loop {
            let processed = self.parse_value()?;
            if !processed {
                break;
            }
            let processed_comma = self.parse_char(',');
            self.parse_whitespace_and_comments();
            if processed_comma {
                self.strip_last_occurrence(',');
            }
            self.skip_semicolons();
            values.push(std::mem::take(&mut self.output));
            self.output = String::new();
        }

        self.parse_whitespace_and_comments();

        if !self.at_end() {
            return Err(self.error_char("Unexpected character"));
        }

        if values.len() == 1 {
            self.output = values.into_iter().next().unwrap_or_default();
        } else {
            self.output = format!("[\n{}\n]", values.join(",\n"));
        }
        Ok(self.output)
    }

    fn is_jsonp_close(&self) -> bool {
        self.peek() == Some(')')
    }

    fn skip_semicolons(&mut self) {
        while self.peek() == Some(';') {
            self.pos += 1;
            self.parse_whitespace_and_comments();
        }
    }
}
