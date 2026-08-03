//! Recognition and parsing of the streaming-response shape.
//!
//! ## The shape
//!
//! ```text
//! {
//!   stream: fn() { ... },        // required, zero-argument generator
//!   status: 200,                 // optional, default 200
//!   headers: { ... },            // optional; SSE defaults are pre-seeded
//!   max_events: 100,             // optional bound
//!   max_duration_ms: 30000,      // optional bound
//!   chunked: false,              // optional transfer coding selector
//! }
//! ```
//!
//! ## Why `stream` must hold a callable
//!
//! [`is_stream`] deliberately requires *both* the reserved key and a function
//! value. The ordinary response path (`http_response_extract`) accepts `status`, `headers`, and `body` only, and stringifies anything
//! else it finds under `body`. A function is the one value that path can never
//! render usefully, so a callable under `stream` cannot be an accident: no
//! existing handler produces it, and no future handler produces it by mistake.
//! A `stream` key holding a non-callable is rejected loudly by [`parse`] rather
//! than silently falling back, so a typo surfaces as an error naming the key.

use std::collections::HashMap;

use crate::value::Value;

use super::bounds::Bounds;
use super::chunk::Coding;
use super::fields;

/// A validated streaming response, ready for [`super::head`] and [`super::pump`].
pub(crate) struct StreamSpec {
    /// Status code for the response head.
    pub(crate) status: u16,
    /// Response headers, with SSE defaults already applied.
    pub(crate) headers: HashMap<String, String>,
    /// Zero-argument generator invoked once per event.
    pub(crate) generator: Value,
    /// Event-count and duration caps.
    pub(crate) bounds: Bounds,
    /// Body transfer coding.
    pub(crate) coding: Coding,
}

/// Report whether `resp` is a streaming response rather than an ordinary one.
///
/// # Arguments
///
/// * `resp` — The value a handler returned.
///
/// # Returns
///
/// `true` when `resp` is a map whose `stream` key holds a callable. Infallible;
/// it never inspects anything else, so it is safe to call on every response.
///
/// # Examples
///
/// ```text
/// is_stream(&Value::Str(..))                       == false
/// is_stream(&map_with("body", str))                == false
/// is_stream(&map_with("stream", Value::Fn(..)))    == true
/// is_stream(&map_with("stream", Value::Int(1)))    == false // parse() then errors
/// ```
pub(crate) fn is_stream(resp: &Value) -> bool {
    let Value::Map(map) = resp else {
        return false;
    };
    matches!(
        map.borrow().get("stream"),
        Some(Value::Fn(_) | Value::VmFn(_) | Value::Native(_))
    )
}

/// Validate a streaming response map into a [`StreamSpec`].
///
/// # Arguments
///
/// * `resp` — The handler's return value.
///
/// # Returns
///
/// The parsed specification.
///
/// # Errors
///
/// Returns `Err` naming the offending key when `resp` is not a map, when
/// `stream` is missing or not callable, when `status` is not an int in
/// `100..=599`, or when either bound is not a positive int.
pub(crate) fn parse(resp: &Value) -> Result<StreamSpec, String> {
    let Value::Map(map) = resp else {
        return Err(format!(
            "http_serve: streaming response must be a map, got {}",
            resp.type_name()
        ));
    };
    let map = map.borrow();
    Ok(StreamSpec {
        status: fields::status(&map)?,
        headers: fields::headers(&map),
        generator: fields::generator(&map)?,
        bounds: Bounds::parse(&map)?,
        coding: Coding::parse(&map)?,
    })
}
