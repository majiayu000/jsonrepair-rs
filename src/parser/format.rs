use crate::chars;

use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_markdown_fenced(&mut self) -> Result<bool> {
        if !self.matches_at(self.pos, "```") {
            return Ok(false);
        }

        self.enter_container()?;
        self.pos += 3; // skip opening ```

        // Optional language tag: ```json
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                self.pos += 1;
            } else {
                break;
            }
        }

        self.parse_whitespace_and_comments();

        let processed_value = self.parse_value(false)?;
        if !processed_value {
            self.leave_container();
            return Ok(false);
        }

        self.parse_whitespace_and_comments();

        if self.matches_at(self.pos, "```") {
            self.pos += 3;
        }

        self.leave_container();
        Ok(true)
    }

    pub(super) fn parse_markdown_wrapped_open(&mut self) -> bool {
        self.parse_whitespace_and_comments();
        for block in ["```", "[```", "{```"] {
            if self.matches_at(self.pos, block) {
                self.pos += block.len();

                // Optional language tag like ```json
                while self.peek().is_some_and(chars::is_identifier_char) {
                    self.pos += 1;
                }

                self.parse_whitespace_and_comments();
                return true;
            }
        }
        false
    }

    pub(super) fn parse_markdown_wrapped_close(&mut self) -> bool {
        self.parse_whitespace_and_comments();
        for block in ["```", "```]", "```}"] {
            if self.matches_at(self.pos, block) {
                self.pos += block.len();
                self.parse_whitespace_and_comments();
                return true;
            }
        }
        false
    }

    pub(super) fn parse_regex_as_string(&mut self, is_wrapper_argument: bool) -> Result<bool> {
        if self.peek() != Some('/') {
            return Ok(false);
        }
        self.pos += 1;
        self.output.push('"');
        self.output.push('/');
        let mut escaped = false;
        let mut parenthesis_depth = 0usize;
        let mut in_character_class = false;
        let mut wrapper_boundary = None;

        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => {
                    if let Some((input_end, output_end)) = wrapper_boundary {
                        // Only a missing slash permits repairing before the wrapper close.
                        self.pos = input_end;
                        self.output.truncate(output_end);
                    }
                    self.output.push('/');
                    self.output.push('"');
                    return Ok(true);
                }
                Some(c) => {
                    if is_wrapper_argument && !escaped && wrapper_boundary.is_none() {
                        match c {
                            '[' => in_character_class = true,
                            ']' => in_character_class = false,
                            '(' if !in_character_class => parenthesis_depth += 1,
                            ')' if !in_character_class => {
                                if parenthesis_depth > 0 {
                                    parenthesis_depth -= 1;
                                } else {
                                    // Keep scanning when regex data follows this candidate.
                                    let next =
                                        self.chars[self.pos + 1..].iter().copied().find(|&ch| {
                                            !chars::is_whitespace(ch) || matches!(ch, '\n' | '\r')
                                        });
                                    if next.map_or(true, |ch| {
                                        chars::is_unquoted_string_delimiter(ch)
                                            || matches!(ch, ')' | '#')
                                    }) {
                                        wrapper_boundary = Some((self.pos, self.output.len()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.pos += 1;
                    if c == '/' && !escaped {
                        self.output.push('/');
                        while self.peek().is_some_and(|flag| flag.is_ascii_alphabetic()) {
                            let flag = self.chars[self.pos];
                            self.push_string_char(flag);
                            self.pos += 1;
                        }
                        self.output.push('"');
                        return Ok(true);
                    }

                    self.push_string_char(c);
                    escaped = c == '\\' && !escaped;
                }
            }
        }
    }
}
