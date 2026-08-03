//! The individual header checks behind [`validate_request`].
//!
//! `Connection` is matched per token, not by substring, because it is a
//! comma-separated list and a real browser sends `keep-alive, Upgrade`. A naive
//! equality check would reject conforming clients; a naive `contains` would
//! accept a header like `X-Upgrade-Nothing`.

use crate::websocket::handshake_error::HandshakeError;
use crate::websocket::handshake_key::check_key;

/// Verify the four required headers and return the validated key.
///
/// # Arguments
///
/// * `header` — Case-insensitive header lookup.
///
/// # Returns
///
/// The `Sec-WebSocket-Key` value, trimmed and proven to be 16 base64 bytes.
///
/// # Errors
///
/// A [`HandshakeError`] naming the first header that failed.
pub(super) fn check_headers<'a, F>(header: &F) -> Result<&'a str, HandshakeError>
where
    F: Fn(&str) -> Option<&'a str>,
{
    let upgrade = required(header, "upgrade")?;
    if !upgrade.trim().eq_ignore_ascii_case("websocket") {
        return Err(HandshakeError::BadUpgrade {
            value: upgrade.to_string(),
        });
    }
    let connection = required(header, "connection")?;
    let has_upgrade = connection
        .split(',')
        .any(|token| token.trim().eq_ignore_ascii_case("upgrade"));
    if !has_upgrade {
        return Err(HandshakeError::BadConnection {
            value: connection.to_string(),
        });
    }
    let version = required(header, "sec-websocket-version")?;
    if version.trim() != "13" {
        return Err(HandshakeError::UnsupportedVersion {
            version: version.to_string(),
        });
    }
    let key = required(header, "sec-websocket-key")?.trim();
    check_key(key)?;
    Ok(key)
}

/// Fetch a header that must be present.
///
/// # Arguments
///
/// * `header` — Case-insensitive header lookup.
/// * `name` — The lowercased field name to require.
///
/// # Returns
///
/// The field value.
///
/// # Errors
///
/// [`HandshakeError::MissingHeader`] naming `name`.
fn required<'a, F>(header: &F, name: &'static str) -> Result<&'a str, HandshakeError>
where
    F: Fn(&str) -> Option<&'a str>,
{
    header(name).ok_or(HandshakeError::MissingHeader { name })
}
