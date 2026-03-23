use super::JsonRepairer;
use super::Result;

impl JsonRepairer {
    pub(super) fn parse_markdown_fenced(&mut self) -> Result<bool> {
        self.pos += 3; // skip opening ```
                       // Skip optional language tag
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                self.pos += 1;
                break;
            }
            self.pos += 1;
        }

        let content_start = self.pos;
        let mut content_end = self.chars.len();
        while self.pos < self.chars.len() {
            if self.matches_at(self.pos, "```") {
                content_end = self.pos;
                self.pos += 3;
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        self.pos += 1;
                        break;
                    }
                    self.pos += 1;
                }
                break;
            }
            self.pos += 1;
        }

        let content: String = self.chars[content_start..content_end].iter().collect();
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }

        let inner = JsonRepairer::new(trimmed);
        match inner.repair() {
            Ok(repaired) => {
                self.output.push_str(&repaired);
                Ok(true)
            }
            Err(_) => {
                self.output.push('"');
                for c in trimmed.chars() {
                    match c {
                        '"' => self.output.push_str("\\\""),
                        '\\' => self.output.push_str("\\\\"),
                        '\n' => self.output.push_str("\\n"),
                        _ => self.output.push(c),
                    }
                }
                self.output.push('"');
                Ok(true)
            }
        }
    }

    pub(super) fn parse_regex_as_string(&mut self) -> Result<bool> {
        if self.peek() != Some('/') {
            return Ok(false);
        }
        self.pos += 1;
        self.output.push('"');
        self.output.push('/');

        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => {
                    self.output.push('/');
                    self.output.push('"');
                    return Ok(true);
                }
                Some('/') => {
                    self.pos += 1;
                    self.output.push('/');
                    while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                        self.output.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                    self.output.push('"');
                    return Ok(true);
                }
                Some('\\') => {
                    self.pos += 1;
                    self.output.push_str("\\\\");
                    if let Some(c) = self.peek() {
                        self.pos += 1;
                        if c == '"' {
                            self.output.push_str("\\\"");
                        } else {
                            self.output.push(c);
                        }
                    }
                }
                Some(c) => {
                    self.pos += 1;
                    if c == '"' {
                        self.output.push_str("\\\"");
                    } else {
                        self.output.push(c);
                    }
                }
            }
        }
    }
}
