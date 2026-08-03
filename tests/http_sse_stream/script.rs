//! The tetherscript program under test.
//!
//! One program serves every route so a single spawned binary covers all
//! assertions. `/health` is deliberately an *ordinary* response: the contrast is
//! what proves the streaming shape does not capture normal handlers.
//!
//! The source is assembled from two halves — [`generators::SOURCE`] and
//! [`routes::SOURCE`] — purely so neither file exceeds the repository's 50-line
//! limit. The split is at a function boundary, so each half is readable alone.

#[path = "script_generators.rs"]
mod generators;
#[path = "script_routes.rs"]
mod routes;

/// Routes exercised by the tests in this suite.
///
/// | Path | Shape |
/// | --- | --- |
/// | `/health` | Ordinary fixed-length response |
/// | `/events` | Three data events, no delay |
/// | `/slow` | Three data events, 150 ms apart |
/// | `/mixed` | A comment, a retry frame, then one event |
/// | `/multiline` | One event whose data spans two lines |
/// | `/chunked` | Two data events under chunked coding |
/// | `/runaway` | A generator that never ends, capped at 4 events |
/// | `/endless` | A generator that never ends, 40 ms per event |
///
/// # Returns
///
/// The complete program text. Infallible; the halves are compile-time constants.
pub(crate) fn source() -> String {
    format!("{}{}", generators::SOURCE, routes::SOURCE)
}
