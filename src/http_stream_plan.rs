//! The shape of a streaming response, resolved once before any event is written.
//!
//! Parsing the response map up front means a malformed `stream` field is reported before a
//! head goes out, rather than mid-body where a client has already committed to reading.

use crate::value::Value;

/// Default bound on events, when the handler names none.
///
/// A generator that never returns nil would otherwise hold the single-threaded accept loop
/// forever, starving every other request.
const DEFAULT_MAX_EVENTS: i64 = 10_000;

/// A validated streaming response.
pub(crate) struct Plan {
    /// Callable producing the next event, or nil when finished.
    generator: Value,
    /// Whether to frame with `Transfer-Encoding: chunked`.
    chunked: bool,
    /// Maximum events to pull before stopping.
    max_events: i64,
    /// Status line and headers, already rendered.
    head: String,
}

impl Plan {
    /// Read the plan out of a handler's response map.
    ///
    /// # Errors
    ///
    /// Returns an error when `stream` is absent or not callable, or when `chunked` or
    /// `max_events` have the wrong type.
    pub(crate) fn from_response(resp: &Value) -> Result<Self, String> {
        let Value::Map(fields) = resp else {
            return Err("http_serve: a streaming response must be a map".to_string());
        };
        let fields = fields.borrow();
        let generator = fields
            .get("stream")
            .cloned()
            .ok_or("http_serve: streaming response has no `stream` field")?;
        let chunked = super::http_stream_fields::flag(&fields, "chunked")?;
        let max_events =
            super::http_stream_fields::count(&fields, "max_events", DEFAULT_MAX_EVENTS)?;
        let status = super::http_stream_fields::status(&fields)?;
        let head = super::http_stream_head::render(status, &fields, chunked);
        Ok(Self {
            generator,
            chunked,
            max_events,
            head,
        })
    }

    /// The rendered status line and headers.
    pub(crate) fn head(&self) -> &str {
        &self.head
    }

    /// The generator to call per event.
    pub(crate) fn generator(&self) -> &Value {
        &self.generator
    }

    /// Whether chunked framing applies.
    pub(crate) fn chunked(&self) -> bool {
        self.chunked
    }

    /// The event bound.
    pub(crate) fn max_events(&self) -> i64 {
        self.max_events
    }
}
