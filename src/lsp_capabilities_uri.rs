//! `file:` URI ↔ filesystem path conversion.
//!
//! Cross-module go-to-definition needs a real path: the client sends a
//! `file:` URI, imports are resolved relative to the *importing file's*
//! directory (see [`crate::modules`]), and the reply must be a URI again.
//! Nothing in the crate did this conversion before, because the diagnostics-only
//! server never had to leave the document it was handed.
//!
//! This is deliberately a minimal, dependency-free conversion covering what
//! editors actually send: `file:///abs/path` on Unix, `file:///C:/path` on
//! Windows, and percent-encoded bytes. Non-`file:` schemes yield `None`, which
//! callers turn into a null definition result rather than an error, because a
//! document in an untitled or virtual buffer legitimately has no path.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::uri::{to_path, from_path};
//! use std::path::Path;
//!
//! assert_eq!(to_path("file:///tmp/a%20b.tether"), Some("/tmp/a b.tether".into()));
//! assert!(to_path("untitled:Untitled-1").is_none());
//! assert!(from_path(Path::new("/tmp/x.tether")).starts_with("file:///"));
//! ```

use std::path::{Path, PathBuf};

/// Convert a `file:` URI into a filesystem path.
///
/// # Arguments
///
/// * `uri` — URI as sent by the client.
///
/// # Returns
///
/// `Some(path)` for a `file:` URI, `None` for any other scheme.
///
/// # Errors
///
/// Infallible; unsupported schemes are reported as `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::uri::to_path;
/// assert_eq!(to_path("file:///a/b.tether"), Some("/a/b.tether".into()));
/// assert_eq!(to_path("file:///C:/a.tether"), Some("C:/a.tether".into()));
/// ```
pub fn to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.split_once('/').map(|(_, tail)| tail).unwrap_or(rest);
    let decoded = decode(rest);
    let trimmed = strip_drive_slash(&decoded);
    Some(PathBuf::from(trimmed))
}

fn strip_drive_slash(path: &str) -> String {
    let bytes = path.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if drive {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        match escape_at(bytes, index) {
            Some(byte) => {
                out.push(byte);
                index += 3;
            }
            None => {
                out.push(bytes[index]);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn escape_at(bytes: &[u8], index: usize) -> Option<u8> {
    if bytes[index] != b'%' || index + 2 >= bytes.len() {
        return None;
    }
    let high = (bytes[index + 1] as char).to_digit(16)?;
    let low = (bytes[index + 2] as char).to_digit(16)?;
    Some((high * 16 + low) as u8)
}

/// Convert a filesystem path into a `file:` URI.
///
/// # Arguments
///
/// * `path` — Absolute path to encode.
///
/// # Returns
///
/// A `file:///...` URI with spaces and `%` percent-encoded. Only those two are
/// escaped: over-escaping breaks client-side URI comparison, and these are the
/// characters that actually appear in paths and would otherwise be ambiguous.
///
/// # Errors
///
/// Infallible; non-UTF-8 path components are replaced lossily.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::uri::from_path;
/// use std::path::Path;
/// assert_eq!(from_path(Path::new("/a/b c.tether")), "file:///a/b%20c.tether");
/// ```
pub fn from_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let escaped = text.replace('%', "%25").replace(' ', "%20");
    format!("file:///{}", escaped.trim_start_matches('/'))
}
