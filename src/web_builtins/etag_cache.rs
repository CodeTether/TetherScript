//! `Cache-Control` header construction.
//!
//! Directive order follows the reference middleware in the reference application
//! (`src/middleware/cache_headers.rs`), which emits values such as
//! `public, max-age=31536000, immutable` and `private, no-cache, must-revalidate`.
//! Order is fixed rather than map-iteration dependent, so output is stable.

use std::collections::HashMap;

use super::etag_options::{flag, seconds};
use crate::value::Value;

/// Build a `Cache-Control` value from an options map.
///
/// # Arguments
///
/// * `opts` — Any of `public`, `private`, `no_store`, `no_cache`,
///   `must_revalidate`, `immutable` (bools), plus `max_age` and `s_maxage`
///   (ints, in seconds).
///
/// # Returns
///
/// The header value. An empty options map yields `no-store`, the safe default:
/// returning an empty header would let an intermediary apply its own heuristic
/// caching to a response the caller never classified.
///
/// # Errors
///
/// Returns an error when a value has the wrong type, when both `public` and
/// `private` are set, or when `no_store` is combined with a freshness lifetime.
/// `no_store` forbids storing the response at all, so pairing it with `max_age`
/// is contradictory and is rejected rather than emitted silently.
///
/// # Examples
///
/// ```tether
/// let opts = map()
/// opts.public = true
/// opts.max_age = 31536000
/// opts.immutable = true
/// println(cache_control(opts)?)   // public, max-age=31536000, immutable
/// ```
pub(super) fn build(opts: &HashMap<String, Value>) -> Result<String, String> {
    let mut parts: Vec<String> = Vec::new();
    let (public, private) = (flag(opts, "public")?, flag(opts, "private")?);
    if public && private {
        return Err("cache_control: `public` and `private` are mutually exclusive".into());
    }
    if public {
        parts.push("public".into());
    }
    if private {
        parts.push("private".into());
    }

    let no_store = flag(opts, "no_store")?;
    let max_age = seconds(opts, "max_age")?;
    let s_maxage = seconds(opts, "s_maxage")?;
    if no_store && (max_age.is_some() || s_maxage.is_some()) {
        return Err(
            "cache_control: `no_store` cannot be combined with `max_age` or `s_maxage`".into(),
        );
    }
    if no_store {
        parts.push("no-store".into());
    }
    if flag(opts, "no_cache")? {
        parts.push("no-cache".into());
    }
    if let Some(value) = max_age {
        parts.push(format!("max-age={value}"));
    }
    if let Some(value) = s_maxage {
        parts.push(format!("s-maxage={value}"));
    }
    if flag(opts, "must_revalidate")? {
        parts.push("must-revalidate".into());
    }
    if flag(opts, "immutable")? {
        parts.push("immutable".into());
    }

    if parts.is_empty() {
        return Ok("no-store".into());
    }
    Ok(parts.join(", "))
}
