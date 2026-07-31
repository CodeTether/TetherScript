//! Date and time built-ins.
//!
//! Cookies need an `Expires` value in RFC 7231 IMF-fixdate form, ETags need
//! `Last-Modified`, and JWT `exp`/`iat` are Unix seconds. Nothing in-tree could
//! format or parse either, so the port could not set a real session expiry.
//!
//! Implemented with no dependency: `days_from_civil`/`civil_from_days` carry the
//! full Gregorian leap rule and the weekday is computed from the day count rather
//! than guessed, because an off-by-one here silently writes cookie expiries into
//! the wrong year.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `http_date(unix_seconds)` | `Result` of `Wed, 21 Oct 2015 07:28:00 GMT` |
//! | `http_date_parse(text)` | `Result` of Unix seconds |
//! | `rfc3339(unix_seconds)` | `Result` of `2015-10-21T07:28:00Z` |
//! | `rfc3339_parse(text)` | `Result` of Unix seconds |
//! | `time_now_secs()` | Unix seconds |
//!
//! # Examples
//!
//! ```tether
//! let opts = map()
//! opts.expires = http_date(time_now_secs() + 604800).unwrap()
//! println(cookie_serialize("sid", "abc", opts).unwrap())
//! ```
//!
//! # Reconstruction note
//!
//! This entry point was rebuilt by the integrator after a parallel agent deleted
//! it; the concern modules below are the original implementation.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "datetime_args.rs"]
pub(super) mod datetime_args;
#[path = "datetime_civil.rs"]
pub(super) mod datetime_civil;
#[path = "datetime_fields.rs"]
pub(super) mod datetime_fields;
#[path = "datetime_format.rs"]
pub(super) mod datetime_format;
#[path = "datetime_http.rs"]
pub(super) mod datetime_http;
#[path = "datetime_install.rs"]
pub(super) mod datetime_install;
#[path = "datetime_month.rs"]
pub(super) mod datetime_month;
#[path = "datetime_parse.rs"]
pub(super) mod datetime_parse;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    datetime_install::install(env);
}
