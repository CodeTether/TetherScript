//! Token-bucket rate limiting.
//!
//! `http_serve` runs a single-threaded accept loop, so one client issuing requests
//! in a tight loop starves every other client. Throttling is therefore an
//! availability gate for the port, not a nicety.
//!
//! # The caller owns the state
//!
//! `bucket_take` does not mutate the bucket it is given; it returns the next one
//! alongside the decision. Nothing here holds global mutable state, which keeps
//! the limiter directly testable and lets a script key buckets however it likes —
//! per IP, per API key, per route — by storing them in an ordinary map.
//!
//! **The caller must persist the returned `bucket`.** Discarding it silently
//! disables the limiter, because every call would then start from a full bucket.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `bucket_new(capacity, refill_per_sec)` | `Result` of a bucket map |
//! | `bucket_take(bucket, cost)` | `Result` of `allowed` / `bucket` / `retry_after_ms` |
//! | `retry_after_header(retry_after_ms)` | int seconds, rounded up |
//! | `too_many_requests_response(retry_after_ms)` | map with status 429 |
//!
//! # Examples
//!
//! ```tether
//! let mut b = bucket_new(10, 2)?
//! let took = bucket_take(b, 1)?
//! b = took.bucket                      // persist, or the limit never applies
//!
//! if !took.allowed {
//!     return too_many_requests_response(took.retry_after_ms)
//! }
//! ```
//!
//! # Layout
//!
//! * `ratelimit_bucket` — field names, the clock, and the refill arithmetic
//! * `ratelimit_take` — construction and the take decision
//! * `ratelimit_shape` — the bucket and outcome map shapes
//! * `ratelimit_fields` — numeric field coercion with named errors
//! * `ratelimit_response` — `Retry-After` and the 429 response
//! * `ratelimit_install` — built-in registration
//! * `ratelimit_arg` — bucket argument borrowing

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "ratelimit_arg.rs"]
mod ratelimit_arg;
#[path = "ratelimit_bucket.rs"]
mod ratelimit_bucket;
#[path = "ratelimit_fields.rs"]
mod ratelimit_fields;
#[path = "ratelimit_install.rs"]
mod ratelimit_install;
#[path = "ratelimit_response.rs"]
mod ratelimit_response;
#[path = "ratelimit_shape.rs"]
mod ratelimit_shape;
#[path = "ratelimit_take.rs"]
mod ratelimit_take;

/// Register this group's built-ins.
///
/// Defines `bucket_new`, `bucket_take`, `retry_after_header`, and
/// `too_many_requests_response` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    ratelimit_install::install(env);
}
