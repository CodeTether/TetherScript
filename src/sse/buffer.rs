//! Buffer ownership for [`EventStream`]: append, inspect, drain.
//!
//! Deliberately separate from [`super::send`]: this module knows nothing about
//! SSE syntax and only moves bytes, so the framing rules and the memory rules can
//! be reviewed independently.

use super::EventStream;

impl EventStream {
    /// Append already-rendered bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` — Raw frame bytes. **Not** validated: only pass output from
    ///   [`super::Event::render`], [`super::fields`], or [`super::validate`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.push_raw(b"data: x\n\n");
    /// assert_eq!(stream.buffered(), 9);
    /// ```
    pub fn push_raw(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Borrow the pending bytes without consuming them.
    ///
    /// # Returns
    ///
    /// Everything framed since the last [`EventStream::take`], in wire order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_data("x");
    /// assert_eq!(stream.as_bytes(), b"data: x\n\n");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Number of bytes currently buffered.
    ///
    /// # Returns
    ///
    /// The pending byte count, which is what [`EventStream::should_drop`]
    /// compares against [`EventStream::bound`].
    pub fn buffered(&self) -> usize {
        self.buf.len()
    }

    /// Whether nothing is pending.
    ///
    /// # Returns
    ///
    /// `true` when the buffer is empty — that is, when the transport is caught up.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert!(tetherscript::sse::EventStream::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// The configured buffer bound in bytes.
    ///
    /// # Returns
    ///
    /// The bound set at construction; see [`super::backpressure`].
    pub fn bound(&self) -> usize {
        self.bound
    }

    /// Drain the buffer, handing ownership of the pending bytes to the caller.
    ///
    /// Call this immediately before writing to the socket. The stream is left
    /// empty, so a partial write must be retried by the caller — this type does
    /// not remember what it gave away.
    ///
    /// # Returns
    ///
    /// The pending bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_data("x");
    /// assert_eq!(stream.take(), b"data: x\n\n".to_vec());
    /// assert!(stream.is_empty());
    /// ```
    pub fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buf)
    }
}
