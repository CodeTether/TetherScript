//! # Byte cursor over a receive buffer
//!
//! RESP is CRLF-delimited rather than length-prefixed at the frame level, so
//! decoding is a walk over a borrowed buffer that must be able to stop and say
//! "not yet" at any point. [`Cursor`] provides exactly that: every accessor
//! returns [`DecodeError::Incomplete`] when the bytes it needs have not arrived,
//! and the position advances only on success.
//!
//! The cursor never mutates or takes ownership of the caller's buffer. The
//! consumed-byte count that [`super::decode`] returns is simply
//! [`Cursor::position`] at the end of a successful parse, which is what lets a
//! client drain its buffer only for the replies it actually received.

use super::crlf;
use super::error::DecodeError;

/// A read-only position within a receive buffer.
pub(super) struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Start at offset zero of `buf`.
    pub(super) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Bytes consumed so far.
    pub(super) fn position(&self) -> usize {
        self.pos
    }

    /// Read one byte and advance.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Incomplete`] when the buffer is exhausted.
    pub(super) fn byte(&mut self) -> Result<u8, DecodeError> {
        let byte = *self.buf.get(self.pos).ok_or(DecodeError::Incomplete)?;
        self.pos += 1;
        Ok(byte)
    }

    /// Read `count` bytes and advance.
    ///
    /// Used for bulk payloads, so it deliberately does not inspect what it hands
    /// back: the length prefix is authoritative and the payload may contain CRLF
    /// or arbitrary binary.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Incomplete`] when fewer than `count` bytes remain.
    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.pos.checked_add(count).ok_or(DecodeError::Incomplete)?;
        let slice = self.buf.get(self.pos..end).ok_or(DecodeError::Incomplete)?;
        self.pos = end;
        Ok(slice)
    }

    /// Consume the CRLF that terminates a bulk or verbatim payload.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Incomplete`] when fewer than two bytes remain;
    /// [`DecodeError::Malformed`] when those two bytes are not `\r\n`, meaning the
    /// announced payload length disagreed with the framing.
    pub(super) fn crlf(&mut self) -> Result<(), DecodeError> {
        match self.take(2)? {
            b"\r\n" => Ok(()),
            other => Err(DecodeError::malformed(format!(
                "expected CRLF after payload, found {other:?}"
            ))),
        }
    }

    /// Read one CRLF-terminated line, returning it without the CRLF.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Incomplete`] when no CRLF has arrived yet;
    /// [`DecodeError::Malformed`] once the unterminated run passes the line bound.
    /// See [`crlf::find`] for that decision.
    pub(super) fn line(&mut self) -> Result<&'a [u8], DecodeError> {
        let rest = self.buf.get(self.pos..).ok_or(DecodeError::Incomplete)?;
        let len = crlf::find(rest)?;
        let line = self.take(len)?;
        self.crlf()?;
        Ok(line)
    }
}
