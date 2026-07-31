//! MIME type built-ins.
//!
//! Removes the per-script content-type guessing: `examples/static_site_server.tether`
//! hand-rolls a four-branch `content_type` that falls back to plain text for
//! everything unrecognized, so it currently mislabels every image, font, and PDF
//! it serves.
//!
//! The shared extensions are copied verbatim from `crate::http_static`'s table so
//! the built-in and the native static server can never disagree about a file.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `mime_for_path(path)` | content-type str, `application/octet-stream` when unknown |
//! | `mime_parse(header)` | `Result` of a map: `type` plus one key per parameter |
//! | `mime_is_text(content_type)` | bool |
//!
//! # Examples
//!
//! ```tether
//! println(mime_for_path("logo.png"))                 // image/png
//! println(mime_parse("text/html; charset=utf-8").unwrap().charset)
//! ```
//!
//! # Reconstruction note
//!
//! This entry point was rebuilt by the integrator after a parallel agent deleted
//! it; the concern modules below are the original implementation.

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::{Env, Value};

use super::super::pure_native;

#[path = "mime_header.rs"]
pub(super) mod mime_header;
#[path = "mime_table.rs"]
pub(super) mod mime_table;
#[path = "mime_text.rs"]
pub(super) mod mime_text;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "mime_for_path",
        pure_native("mime_for_path", Some(1), |args| {
            let path = str_arg(&args[0], "mime_for_path: path")?;
            Ok(Value::Str(Rc::new(mime_table::for_path(&path).to_string())))
        }),
        false,
    );
    bindings.define(
        "mime_parse",
        pure_native("mime_parse", Some(1), |args| {
            let header = str_arg(&args[0], "mime_parse: header")?;
            Ok(result_value(mime_header::parse(&header)))
        }),
        false,
    );
    bindings.define(
        "mime_is_text",
        pure_native("mime_is_text", Some(1), |args| {
            let content_type = str_arg(&args[0], "mime_is_text: content_type")?;
            Ok(Value::Bool(mime_text::is_text(&content_type)))
        }),
        false,
    );
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
