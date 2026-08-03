//! Framing helpers on [`EventStream`]: turn a value into a frame and buffer it.
//!
//! Every method here delegates its syntax to [`super::event`], [`super::fields`],
//! or [`super::validate`] and its byte handling to [`super::buffer`]. Nothing new
//! about the wire format is decided in this file.

use super::error::SseError;
use super::event::Event;
use super::fields::{comment, retry_line};
use super::EventStream;

impl EventStream {
    /// Frame a full [`Event`] and buffer it.
    ///
    /// # Arguments
    ///
    /// * `event` — The event to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` once buffered. Nothing is buffered when validation fails, so a
    /// rejected event cannot leave a half-written frame on the wire.
    ///
    /// # Errors
    ///
    /// Propagates [`SseError`] from [`Event::render`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::{Event, EventStream};
    ///
    /// let mut stream = EventStream::new();
    /// stream.send(&Event::data("hi").name("tick")).unwrap();
    /// assert_eq!(stream.as_bytes(), b"event: tick\ndata: hi\n\n");
    /// ```
    pub fn send(&mut self, event: &Event) -> Result<(), SseError> {
        let frame = event.render()?;
        self.push_raw(frame.as_bytes());
        Ok(())
    }

    /// Frame an unnamed `data`-only event.
    ///
    /// # Arguments
    ///
    /// * `payload` — Message body; newlines become separate `data:` lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_data("a\nb");
    /// assert_eq!(stream.as_bytes(), b"data: a\ndata: b\n\n");
    /// ```
    pub fn send_data(&mut self, payload: &str) {
        // Infallible: a data-only event has no single-line field to reject.
        let frame = Event::data(payload)
            .render()
            .expect("data-only event cannot fail validation");
        self.push_raw(frame.as_bytes());
    }

    /// Buffer a bare `retry:` frame.
    ///
    /// # Arguments
    ///
    /// * `ms` — Reconnection delay in **milliseconds**, an integer per the spec.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_retry(2500);
    /// assert_eq!(stream.as_bytes(), b"retry: 2500\n\n");
    /// ```
    pub fn send_retry(&mut self, ms: u64) {
        self.push_raw(retry_line(ms).as_bytes());
        self.push_raw(b"\n");
    }

    /// Buffer a comment line.
    ///
    /// # Arguments
    ///
    /// * `text` — Comment body, one line.
    ///
    /// # Errors
    ///
    /// [`SseError::MultiLineField`] when `text` spans lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_comment("hello").unwrap();
    /// assert_eq!(stream.as_bytes(), b": hello\n");
    /// ```
    pub fn send_comment(&mut self, text: &str) -> Result<(), SseError> {
        let line = comment(text)?;
        self.push_raw(line.as_bytes());
        Ok(())
    }

    /// Buffer the canonical `: ping` keepalive.
    ///
    /// A comment dispatches nothing at the client; its only job is to put bytes on
    /// the socket so an idle proxy does not reap the connection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::new();
    /// stream.send_keepalive();
    /// assert_eq!(stream.as_bytes(), b": ping\n");
    /// ```
    pub fn send_keepalive(&mut self) {
        self.push_raw(b": ping\n");
    }
}
