//! Cursor over a decoded backend message body.
//!
//! Field layouts are positional, so every accessor advances the cursor and
//! reports a named error rather than panicking on a truncated message.

pub(super) struct Cursor<'a> {
    body: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(body: &'a [u8]) -> Self {
        Self { body, pos: 0 }
    }

    pub(super) fn i32(&mut self) -> Result<i32, String> {
        let slice = self.take(4)?;
        Ok(i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    pub(super) fn i16(&mut self) -> Result<i16, String> {
        let slice = self.take(2)?;
        Ok(i16::from_be_bytes([slice[0], slice[1]]))
    }

    /// Read a NUL-terminated string.
    pub(super) fn cstr(&mut self) -> Result<String, String> {
        let start = self.pos;
        while self.pos < self.body.len() && self.body[self.pos] != 0 {
            self.pos += 1;
        }
        if self.pos >= self.body.len() {
            return Err("postgres: unterminated string in backend message".into());
        }
        let text = String::from_utf8_lossy(&self.body[start..self.pos]).into_owned();
        self.pos += 1;
        Ok(text)
    }

    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self.pos + len;
        let slice = self
            .body
            .get(self.pos..end)
            .ok_or_else(|| format!("postgres: truncated {len}-byte field in backend message"))?;
        self.pos = end;
        Ok(slice)
    }

    pub(super) fn rest(&self) -> &'a [u8] {
        &self.body[self.pos..]
    }
}
