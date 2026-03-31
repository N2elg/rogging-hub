use tracing::warn;

/// Framers JSON objects by tracking brace depth and string escapes.
pub struct JsonFramer {
    pub depth: i32,
    pub in_string: bool,
    pub escape: bool,
    pub start: usize,
}

impl JsonFramer {
    pub fn new() -> Self {
        Self {
            depth: 0,
            in_string: false,
            escape: false,
            start: 0,
        }
    }

    /// Scan `buf` and return absolute (start,end) byte ranges for complete
    /// top-level JSON objects found in the buffer.
    pub fn extract_positions(&mut self, buf: &[u8]) -> Vec<(usize, usize)> {
        let mut objects = Vec::new();

        for (i, &b) in buf.iter().enumerate() {
            if self.in_string {
                if self.escape {
                    self.escape = false;
                    continue;
                }
                match b {
                    b'\\' => self.escape = true,
                    b'"' => self.in_string = false,
                    _ => {}
                }
                continue;
            }

            match b {
                b'"' => {
                    self.in_string = true;
                    self.escape = false;
                }
                b'{' => {
                    if self.depth == 0 {
                        self.start = i;
                    }
                    self.depth += 1;
                }
                b'}' => {
                    self.depth -= 1;
                    if self.depth == 0 {
                        objects.push((self.start, i + 1));
                    }
                    if self.depth < 0 {
                        warn!(offset = i, "Unexpected '}}' while framing JSON, resetting state");
                        self.depth = 0;
                    }
                }
                _ => {}
            }
        }

        objects
    }

    /// Adjust internal `start` after `consumed` bytes were removed from the
    /// front of the buffer.
    pub fn shift(&mut self, consumed: usize) {
        if self.start >= consumed {
            self.start -= consumed;
        } else {
            self.start = 0;
        }
    }
}
