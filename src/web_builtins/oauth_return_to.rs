//! Open-redirect defence for the `return_to` path carried through the flow.
//!
//! # The attack
//!
//! A login flow that remembers where the user was going, and redirects there
//! afterwards, hands an attacker a redirect primitive if the destination is not
//! constrained. `GET /login?return_to=https://evil.example/harvest` yields a link that
//! *starts* on the real, trusted origin — so it survives a glance at the hostname,
//! survives an email filter's domain allowlist, and lands the victim on the attacker's
//! page still believing they are on the real site. That is the classic open redirect,
//! and inside an OAuth flow it is worse than usual: the URL being handed out is the one
//! the user is expected to authenticate through.
//!
//! # Why three separate rejections
//!
//! Checking only for `http://` is the usual bug, and it misses two forms:
//!
//! 1. **Absolute URL** — `http://evil.example`, `https://evil.example`, and any other
//!    `scheme:` prefix. Rejected by requiring a leading `/`.
//! 2. **Scheme-relative** — `//evil.example/path`. This *does* begin with `/`, so a
//!    naive "must start with a slash" check passes it, yet a browser resolves it
//!    against the current scheme and navigates cross-origin. Rejected explicitly.
//! 3. **Backslash forms** — `/\evil.example`, `\\evil.example`, `\/evil.example`.
//!    Browsers normalise `\` to `/` in the authority position, so these are
//!    scheme-relative URLs in disguise that pass both checks above. Any backslash
//!    anywhere is rejected; a legitimate path has no use for one.
//!
//! Control characters and spaces are rejected too: a `return_to` containing `\r` or
//! `\n` becomes a header-injection vector the moment it is written into a `Location`
//! response header.
//!
//! # Examples
//!
//! ```rust,ignore
//! assert!(validate("/dashboard").is_ok());
//! assert!(validate("http://evil.example").is_err());
//! assert!(validate("//evil.example").is_err());
//! assert!(validate("/\\evil.example").is_err());
//! ```

/// Accept only a same-origin, relative destination path.
///
/// # Arguments
///
/// * `path` — Candidate destination, as received from an untrusted request.
///
/// # Returns
///
/// The path unchanged when it is safe to redirect to.
///
/// # Errors
///
/// Returns `Err` naming the rejected form: an empty value, a missing leading `/`, a
/// scheme-relative `//host`, a backslash, or a control or space character.
pub(crate) fn validate(path: &str) -> Result<String, String> {
    reject_shape(path)?;
    reject_characters(path)?;
    Ok(path.to_string())
}

/// Reject absolute and scheme-relative destinations.
fn reject_shape(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("oauth: return_to must not be empty; use \"/\" for the site root".into());
    }
    if !path.starts_with('/') {
        return Err(format!(
            "oauth: return_to `{path}` must be a relative path beginning with `/`; an absolute URL is an open redirect"
        ));
    }
    if path.starts_with("//") {
        return Err(format!(
            "oauth: return_to `{path}` is scheme-relative; `//host` navigates off-origin and is an open redirect"
        ));
    }
    Ok(())
}

/// Reject backslashes and control or space characters.
fn reject_characters(path: &str) -> Result<(), String> {
    if path.contains('\\') {
        return Err(format!(
            "oauth: return_to `{path}` contains a backslash; browsers normalise it to `/`, so this is a disguised off-origin redirect"
        ));
    }
    match path.bytes().find(|byte| *byte < 0x21 || *byte == 0x7f) {
        Some(byte) => Err(format!(
            "oauth: return_to contains the control or space byte 0x{byte:02x}; such a path can inject a response header"
        )),
        None => Ok(()),
    }
}
