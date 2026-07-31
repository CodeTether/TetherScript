//! URL-encoded form and query-string built-ins.
//!
//! Owner: sub-agent `form`. Provides the decoding half the HTTP server was
//! missing: handlers receive `query` and `body` as raw strings (see
//! `src/http_server_request_map.rs`), and nothing could interpret either.
//!
//! This complements `url_parse` rather than duplicating it. `url_parse` splits a
//! URL into scheme/host/path/query and performs no percent-decoding; these
//! built-ins decode and interpret the query or body payload itself.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `url_encode(input)` | str |
//! | `url_decode(input)` | Result of str |
//! | `form_parse(input)` | Result of map |
//! | `form_encode(map)` | Result of str |
//!
//! Fallible built-ins return tetherscript `Result` values, matching the
//! `result_value` convention in `src/system.rs`, so scripts use `?`.
//!
//! # Examples
//!
//! ```tether
//! let fields = form_parse("name=Ada+Lovelace&year=1843")?
//! println(fields.name)             // Ada Lovelace
//! println(url_encode("a b/c"))     // a%20b%2Fc
//! println(url_decode("a%20b")?)    // a b
//! ```
//!
//! # Layout
//!
//! * `form_install` — environment registration
//! * `form_pairs` — `&`/`=` splitting and the map shape
//! * `form_codec` — percent-encode and percent-decode
//! * `form_hex` — hex nibble conversion

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

// Explicit paths: the parent module is declared with `#[path]`, so relative
// submodule resolution would otherwise look in `src/` directly.
#[path = "form_codec.rs"]
mod form_codec;
#[path = "form_hex.rs"]
mod form_hex;
#[path = "form_install.rs"]
mod form_install;
#[path = "form_pairs.rs"]
mod form_pairs;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    form_install::install(env);
}
