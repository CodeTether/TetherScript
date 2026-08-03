//! Key-component validation: the boundary between untrusted text and Redis keys.
//!
//! # Key injection
//!
//! A session id arrives in a cookie, and a rate-limit subject arrives from a header
//! or an API key — both are attacker-controlled strings. Redis keys are flat: the
//! only thing separating `sess:abc` from `ratelimit:abc` is the literal [`SEP`]
//! byte. So an id of `"..:ratelimit:1.2.3.4:60:0"` concatenated into `sess:{id}`
//! addresses the *rate limiter's* key, letting a request read or overwrite another
//! namespace's value. Rejecting [`SEP`] outright is what keeps namespaces disjoint;
//! quoting it would work too, but every backend would then have to agree on the
//! quoting, and disagreement is silent.
//!
//! Control characters are rejected for the same class of reason: they are never
//! legitimate in an id, and they survive into logs and RESP framing.

/// Redis key namespace separator.
///
/// `:` is the near-universal Redis convention (`sess:<id>`), which is why it is the
/// character that must never appear inside a component.
pub(super) const SEP: char = ':';

/// Validate one untrusted key component.
///
/// # Arguments
///
/// * `label` — Built-in and parameter name, used verbatim in the error.
/// * `text` — Candidate component.
///
/// # Returns
///
/// `Ok(())` when `text` is safe to concatenate into a key.
///
/// # Errors
///
/// Returns a named error when `text` is empty, contains [`SEP`], or contains an
/// ASCII control character.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(component("k: id", "9f2c").is_ok());
/// assert!(component("k: id", "a:b").is_err());
/// assert!(component("k: id", "").is_err());
/// ```
pub(super) fn component(label: &str, text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if text.contains(SEP) {
        return Err(format!(
            "{label} must not contain the key separator `{SEP}` (key injection): {text:?}"
        ));
    }
    match text.chars().find(|candidate| candidate.is_control()) {
        Some(bad) => Err(format!(
            "{label} must not contain control character {bad:?}"
        )),
        None => Ok(()),
    }
}
