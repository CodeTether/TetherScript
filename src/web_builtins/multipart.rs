//! `multipart/form-data` built-ins (RFC 7578).
//!
//! File uploads are the one request shape the port cannot handle at all: handlers
//! receive `body` as a raw string (see `src/http_server_request_map.rs`) and
//! nothing could split it. The reference controllers in
//! `the reference application/src/controllers/video_upload.rs` and `design_upload.rs` read a
//! field name and filename off each part, which is the surface provided here.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `multipart_parse(body, boundary)` | `Result` of a list of part maps |
//! | `multipart_field(parts, name)` | `Result` of the first matching body |
//! | `multipart_boundary(content_type)` | `Result` of the boundary str |
//!
//! Each part map carries `name`, `filename`, `content_type`, and `body`. The two
//! optional headers are `nil` when absent, so a file part is distinguishable from
//! a text field, and an empty filename from a missing one.
//!
//! # Body fidelity
//!
//! The delimiter is CRLF + `--` + boundary, and that leading CRLF belongs to the
//! delimiter rather than to the preceding part. A part body is therefore returned
//! byte-exact with no trailing CRLF; treating it otherwise corrupts every upload
//! by two bytes, which is the kind of bug that ships because the file still opens.
//!
//! # Layout
//!
//! * `multipart_install` — environment registration
//! * `multipart_split` — delimiter handling and part extraction
//! * `multipart_headers` — `Content-Disposition` and `Content-Type` per part
//! * `multipart_boundary` — boundary extraction from the request header
//! * `multipart_value` — script-visible maps and field lookup
//!
//! # Examples
//!
//! ```tether
//! let boundary = multipart_boundary(req.headers["content-type"])?
//! let parts = multipart_parse(req.body, boundary)?
//! println(multipart_field(parts, "title")?)
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "multipart_boundary.rs"]
pub(super) mod multipart_boundary;
#[path = "multipart_headers.rs"]
pub(super) mod multipart_headers;
#[path = "multipart_install.rs"]
pub(super) mod multipart_install;
#[path = "multipart_split.rs"]
pub(super) mod multipart_split;
#[path = "multipart_value.rs"]
pub(super) mod multipart_value;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    multipart_install::install(env);
}
