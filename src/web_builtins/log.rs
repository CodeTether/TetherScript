//! Structured logging built-ins.
//!
//! The port could only call `println`, which produces prose no log collector can
//! query. These built-ins emit one JSON object per line, which is how the
//! deployed reference service is actually debugged: its telemetry layer
//! (a telemetry module) configures `.json()` output for the
//! same reason.
//!
//! # Everything goes to stderr
//!
//! Lines are written to **stderr, never stdout**. Stdout is already committed to
//! other protocols: `http_serve` writes response bodies, `PluginHost` captures
//! `println` output through [`crate::output`], and the LSP/JSON-RPC surface
//! speaks framed messages there. A log line on stdout would interleave into an
//! HTTP response body or corrupt a JSON-RPC frame, so it must not go there.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `log_json(level, message, fields)` | `Result` of the emitted line |
//! | `log_debug(message)` | `Result` of the emitted line |
//! | `log_info(message)` | `Result` of the emitted line |
//! | `log_warn(message)` | `Result` of the emitted line |
//! | `log_error(message)` | `Result` of the emitted line |
//! | `log_level_enabled(level)` | `Result` of bool |
//!
//! Every line carries three reserved keys — `level`, `msg`, and a `ts` timestamp
//! in Unix milliseconds. Caller fields are merged around them and can never
//! overwrite them, so a field named `level` cannot relabel the severity of its
//! own line and hide an error.
//!
//! Filtering honours the `LOG_LEVEL` environment variable, defaulting to `info`.
//! An unknown level is a named error rather than a silent default, because a typo
//! that downgraded a call would make the line vanish.
//!
//! # Examples
//!
//! ```tether
//! let fields = map()
//! fields.request_id = uuid_v4()
//! fields.status = 500
//! log_json("error", "upstream timeout", fields)?
//!
//! log_info("server started")?
//! if log_level_enabled("debug")? { log_debug("verbose detail")? }
//! ```
//!
//! # Layout
//!
//! * `log_args` — argument coercion
//! * `log_install` — registration
//! * `log_emit` — stderr emission and `LOG_LEVEL` lookup
//! * `log_line` — reserved keys and JSON encoding
//! * `log_level` — level names, ordering, and filtering

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

// Explicit paths: the parent module is declared with `#[path]`, so relative
// submodule resolution would otherwise look in `src/` directly.
#[path = "log_args.rs"]
mod log_args;
#[path = "log_emit.rs"]
mod log_emit;
#[path = "log_install.rs"]
mod log_install;
#[path = "log_level.rs"]
mod log_level;
#[path = "log_line.rs"]
mod log_line;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    log_install::install(env);
}
