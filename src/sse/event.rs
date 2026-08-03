//! The [`Event`] record: a complete SSE event awaiting rendering.
//!
//! Field order on the wire is fixed as `event`, `id`, `retry`, then `data`.
//! `data` is the only multi-line field, so keeping it last makes a truncated
//! capture obvious at a glance.
//!
//! Building an event never fails; only [`Event::render`] does, so a caller
//! handles rejection in exactly one place.

use super::error::SseError;
use super::fields::{data_lines, retry_line};
use super::validate::{id_line, single_line};

/// One `text/event-stream` event.
///
/// Built with a chained setter style. Every field except `data` is optional, and
/// `data` defaults to empty — an event with an empty payload is legal and
/// dispatches a `message` with an empty string.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::Event;
///
/// let frame = Event::data("hi").name("tick").id("7").retry_ms(1500).render().unwrap();
/// assert_eq!(frame, "event: tick\nid: 7\nretry: 1500\ndata: hi\n\n");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Event {
    /// Payload for the `data:` field(s). Newlines are split on render.
    pub data: String,
    /// Optional `event:` name. Absent means the client fires `message`.
    pub name: Option<String>,
    /// Optional `id:` resume token.
    pub id: Option<String>,
    /// Optional `retry:` reconnection delay, in milliseconds.
    pub retry_ms: Option<u64>,
}

impl Event {
    /// Start an event carrying `payload`.
    ///
    /// # Arguments
    ///
    /// * `payload` — Message body; may span lines.
    ///
    /// # Returns
    ///
    /// An unnamed event with no id and no retry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// assert_eq!(tetherscript::sse::Event::data("x").data, "x");
    /// ```
    pub fn data(payload: impl Into<String>) -> Self {
        Self {
            data: payload.into(),
            ..Self::default()
        }
    }

    /// Set the `event:` name.
    ///
    /// # Arguments
    ///
    /// * `name` — Event type. Must be one line, checked at render time.
    ///
    /// # Returns
    ///
    /// `self`, for chaining.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `id:` resume token.
    ///
    /// # Arguments
    ///
    /// * `id` — Opaque token. Must contain no CR, LF, or NUL; checked at render
    ///   time and rejected, never sanitised.
    ///
    /// # Returns
    ///
    /// `self`, for chaining.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the `retry:` reconnection delay.
    ///
    /// # Arguments
    ///
    /// * `ms` — Delay in **milliseconds**; the spec requires a whole number, so no
    ///   fractional or suffixed form is accepted.
    ///
    /// # Returns
    ///
    /// `self`, for chaining.
    pub fn retry_ms(mut self, ms: u64) -> Self {
        self.retry_ms = Some(ms);
        self
    }

    /// Render the complete frame, blank-line terminator included.
    ///
    /// # Returns
    ///
    /// The frame as a `String`. It **always** ends with `\n\n`: the blank line is
    /// what makes the client dispatch. Omitting it is the single most common SSE
    /// bug — the client buffers the event forever and the page shows nothing, with
    /// no error anywhere to explain it.
    ///
    /// # Errors
    ///
    /// [`SseError::MultiLineField`] when `name` spans lines, or
    /// [`SseError::InvalidId`] when `id` carries CR, LF, or NUL.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::Event;
    ///
    /// assert_eq!(Event::data("a\nb").render().unwrap(), "data: a\ndata: b\n\n");
    /// assert!(Event::data("x").id("bad\nid").render().is_err());
    /// ```
    pub fn render(&self) -> Result<String, SseError> {
        let mut out = String::new();
        if let Some(name) = &self.name {
            out.push_str(&single_line("event", name)?);
        }
        if let Some(id) = &self.id {
            out.push_str(&id_line(id)?);
        }
        if let Some(ms) = self.retry_ms {
            out.push_str(&retry_line(ms));
        }
        out.push_str(&data_lines(&self.data));
        // The blank line dispatches the event. Never make this conditional.
        out.push('\n');
        Ok(out)
    }
}
