//! Test-module registration for the streaming-response implementation.
//!
//! Only declarations and re-exports live here. Grouping them in one child module
//! keeps the parent [`super`] free of a dozen `#[cfg(test)]` blocks, which would
//! otherwise push it past the 50-line limit for reasons unrelated to behaviour.
//!
//! The re-exports below let every child test file reach the implementation as
//! `super::<item>`, so the tests do not care where the parent module is declared.
//!
//! | Module | Concern |
//! | --- | --- |
//! | `values` | Value constructors |
//! | `support` | Fake sinks and a scripted runtime |
//! | `slow` | A runtime with measurable per-call cost |
//! | `shape_tests` | Streaming-vs-ordinary recognition |
//! | `fields` | Header and status defaults |
//! | `bounds_tests` | Bound parsing and the generator contract |
//! | `head_tests` | Response-head bytes |
//! | `wire` | Chunked framing bytes |
//! | `pump_tests` | Flushing and termination |
//! | `starve` | Bound enforcement, protecting the accept loop |
//! | `disconnect` | Client-disconnect handling |

pub(crate) use super::bounds::{
    Bounds, DEFAULT_MAX_DURATION_MS, DEFAULT_MAX_EVENTS, EVENT_CEILING,
};
pub(crate) use super::chunk::{frame_bytes, Coding, TERMINATOR};
pub(crate) use super::head::render as render_head;
pub(crate) use super::pump::payload;
pub(crate) use super::pump::run as run_pump;
pub(crate) use super::shape::{self, StreamSpec};
pub(crate) use super::write::{flush_all, is_disconnect, Flow};
pub(crate) use super::{is_stream, StopReason};

#[path = "http_stream_response_tests_bounds.rs"]
mod bounds_tests;
#[path = "http_stream_response_tests_disconnect.rs"]
mod disconnect;
#[path = "http_stream_response_tests_fields.rs"]
mod fields;
#[path = "http_stream_response_tests_head.rs"]
mod head_tests;
#[path = "http_stream_response_tests_pump.rs"]
mod pump_tests;
#[path = "http_stream_response_tests_shape.rs"]
mod shape_tests;
#[path = "http_stream_response_tests_slow.rs"]
mod slow;
#[path = "http_stream_response_tests_starve.rs"]
mod starve;
#[path = "http_stream_response_tests_support.rs"]
mod support;
#[path = "http_stream_response_tests_values.rs"]
mod values;
#[path = "http_stream_response_tests_wire.rs"]
mod wire;
