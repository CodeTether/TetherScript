//! Server-sent events wire framing (`text/event-stream`).
//!
//! Pure string formatting per the WHATWG HTML `text/event-stream` grammar. There
//! are no sockets and no streaming here: a caller builds a frame with these
//! built-ins and writes it through whatever transport it already owns, so this
//! module stays testable without a server.
//!
//! The detail that matters most is multi-line `data`. A raw newline inside a
//! field value terminates the field, so a two-line payload emitted as one
//! `data:` line would be parsed as a truncated event. Every line therefore gets
//! its own `data:` prefix, and the receiving client rejoins them with `\n`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::Env;

use super::super::pure_native;

#[path = "sse_frame.rs"]
mod frame;

/// Register the SSE framing built-ins.
///
/// Installs `sse_event`, `sse_comment`, and `sse_retry` into `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "sse_event",
        pure_native("sse_event", Some(1), |args| {
            Ok(result_value(frame::event(&args[0])))
        }),
        false,
    );
    bindings.define(
        "sse_comment",
        pure_native("sse_comment", Some(1), |args| {
            Ok(result_value(frame::comment(&args[0])))
        }),
        false,
    );
    bindings.define(
        "sse_retry",
        pure_native("sse_retry", Some(1), |args| {
            Ok(result_value(frame::retry(&args[0])))
        }),
        false,
    );
}
