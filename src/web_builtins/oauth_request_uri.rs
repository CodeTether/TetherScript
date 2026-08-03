//! Shape validation of the `redirect_uri` config field.
//!
//! The exact-match rationale is in [`super`]. This file enforces the shape rules that make
//! an exact comparison meaningful: an absolute `http`/`https` URL, with a host, and with no
//! fragment.
//!
//! A fragment is rejected because RFC 6749 §3.1.2 forbids one, and because a fragment is
//! never sent to the server — so a client that includes one is comparing a different
//! string than the server sees, which defeats the exact comparison the whole defence rests
//! on.
//!
//! `http` is permitted only for a loopback host, matching the OAuth 2.0 Security BCP: a
//! plaintext redirect on a routable host exposes the authorization code to anyone on the
//! network path, while `http://127.0.0.1:PORT/cb` is the standard native-app pattern and
//! never leaves the machine.

/// Hosts for which plaintext `http` is acceptable.
const LOOPBACK: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// Reject a `redirect_uri` that cannot be compared exactly or safely.
///
/// # Arguments
///
/// * `uri` — The configured redirect URI.
/// * `label` — Built-in name, used verbatim in the error message.
///
/// # Returns
///
/// The URI unchanged, so callers pass one value into both the authorization request and
/// the token request and cannot let the two drift apart.
///
/// # Errors
///
/// Returns `Err` when the URI is not absolute `http`/`https`, has no host, carries a
/// `#fragment`, or uses plaintext `http` on a non-loopback host.
pub(crate) fn validate(uri: &str, label: &str) -> Result<String, String> {
    let rest = match (uri.strip_prefix("https://"), uri.strip_prefix("http://")) {
        (Some(rest), _) => rest,
        (None, Some(rest)) => {
            require_loopback(rest, uri, label)?;
            rest
        }
        (None, None) => {
            return Err(format!(
                "{label}: `redirect_uri` `{uri}` must be an absolute http:// or https:// URL"
            ));
        }
    };
    if rest.is_empty() || rest.starts_with('/') {
        return Err(format!("{label}: `redirect_uri` `{uri}` has no host"));
    }
    if uri.contains('#') {
        return Err(format!(
            "{label}: `redirect_uri` `{uri}` must not contain a fragment; the server never sees it, so the exact comparison would fail"
        ));
    }
    Ok(uri.to_string())
}

/// Permit plaintext `http` only for a loopback host.
fn require_loopback(rest: &str, uri: &str, label: &str) -> Result<(), String> {
    let host = host_of(rest);
    if LOOPBACK.contains(&host) {
        return Ok(());
    }
    Err(format!(
        "{label}: `redirect_uri` `{uri}` uses plaintext http on non-loopback host `{host}`; the authorization code would be readable on the network"
    ))
}

/// Extract the host from the part after `scheme://`, keeping a bracketed IPv6 literal
/// intact so `[::1]:8080` is recognised rather than split at its first colon.
fn host_of(rest: &str) -> &str {
    if let Some(end) = rest.strip_prefix('[').and_then(|tail| tail.find(']')) {
        return &rest[..end + 2];
    }
    rest.split(['/', ':', '?', '#']).next().unwrap_or("")
}
