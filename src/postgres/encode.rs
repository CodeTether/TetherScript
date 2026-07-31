//! Byte-level encoding for PostgreSQL frontend messages.
//!
//! The protocol is length-prefixed and big-endian. Every frontend message
//! except the startup packet carries a one-byte tag, then a 4-byte length that
//! counts itself but not the tag.

/// Accumulates a length-prefixed protocol message.
pub(super) struct Builder {
    tag: Option<u8>,
    body: Vec<u8>,
}

impl Builder {
    /// Start a tagged message such as `Q` (query) or `p` (password).
    pub(super) fn tagged(tag: u8) -> Self {
        Self {
            tag: Some(tag),
            body: Vec::new(),
        }
    }

    /// Start an untagged message; only the startup packet uses this form.
    pub(super) fn untagged() -> Self {
        Self {
            tag: None,
            body: Vec::new(),
        }
    }

    pub(super) fn i32(&mut self, value: i32) -> &mut Self {
        self.body.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Append a NUL-terminated string.
    pub(super) fn cstr(&mut self, value: &str) -> &mut Self {
        self.body.extend_from_slice(value.as_bytes());
        self.body.push(0);
        self
    }

    pub(super) fn bytes(&mut self, value: &[u8]) -> &mut Self {
        self.body.extend_from_slice(value);
        self
    }

    /// Finish the message, prefixing the self-inclusive big-endian length.
    pub(super) fn finish(&self) -> Vec<u8> {
        let len = self.body.len() + 4;
        let mut out = Vec::with_capacity(len + 1);
        if let Some(tag) = self.tag {
            out.push(tag);
        }
        out.extend_from_slice(&(len as i32).to_be_bytes());
        out.extend_from_slice(&self.body);
        out
    }
}
