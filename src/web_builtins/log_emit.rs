//! Emitting a rendered log line.
//!
//! Lines go to **stderr**, never stdout. That is not a stylistic choice: the
//! HTTP server writes response bodies to the socket while `println` output is
//! captured by [`crate::output`] for `PluginHost`, and the JSON-RPC/LSP surface
//! speaks a framed protocol on stdout. A log line written to stdout would
//! interleave into a response body or corrupt a JSON-RPC frame, breaking the
//! client rather than merely looking untidy.

use std::io::{self, Write};

use super::{log_level, log_line};
use crate::value::Value;

/// Render a line, write it to stderr, and return it.
///
/// The line is returned as well as emitted so a caller can assert on it, or
/// forward the same bytes to a second sink without re-rendering and risking two
/// slightly different timestamps.
///
/// # Arguments
///
/// * `level` — Severity name.
/// * `message` — Message text.
/// * `fields` — Caller map, or nil.
///
/// # Returns
///
/// The emitted JSON line.
///
/// # Errors
///
/// Returns an error when the level is unknown, the fields are not a map, or
/// encoding fails. A failed write to stderr is deliberately ignored: a closed
/// stderr must not turn logging into a program failure.
pub(super) fn emit(level: &str, message: &str, fields: &Value) -> Result<String, String> {
    let line = log_line::render(level, message, fields)?;
    let mut stderr = io::stderr();
    let _ = writeln!(stderr, "{line}");
    let _ = stderr.flush();
    Ok(line)
}

/// Read the configured threshold from `LOG_LEVEL`.
///
/// # Returns
///
/// The trimmed variable value, or [`log_level::DEFAULT_LEVEL`] when unset or
/// empty. Read per call so a long-running server picks up a change without a
/// restart.
pub(super) fn threshold() -> String {
    match std::env::var("LOG_LEVEL") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => log_level::DEFAULT_LEVEL.to_string(),
    }
}
