//! The set of inputs a page render varies on.
//!
//! # Why this is one type
//!
//! `page_cache_key` and `vary_headers` must agree perfectly about which inputs
//! were consumed: the key says what an entry is *for*, and `Vary` says which
//! request headers a shared cache must compare before reusing it. If they are
//! derived from two separate readings of the caller's map they can drift, and a
//! drift in either direction is a bug — too little `Vary` poisons the cache, too
//! much defeats it. So both built-ins read this one struct.
//!
//! # Fields
//!
//! * `slug` — required, validated by [`super::dynpage_slug`].
//! * `locale` — optional; empty means the render does not depend on language.
//! * `variant` — optional; an A/B assignment.
//! * `device` — optional; a coarse class from [`super::dynpage_device`].
//! * `authenticated` — defaults to false when absent, because false is the
//!   *cacheable* answer and a missing flag must never silently mark a private
//!   render as shareable.

use std::collections::HashMap;

use super::dynpage_args::{map_arg, str_arg};
use super::dynpage_slug;
use crate::value::Value;

/// The validated render inputs.
pub(super) struct Parts {
    /// Normalised page slug.
    pub(super) slug: String,
    /// Negotiated locale, or empty when the render is language-independent.
    pub(super) locale: String,
    /// A/B variant name, or empty when there is no experiment.
    pub(super) variant: String,
    /// Coarse device class, or empty when the render is device-independent.
    pub(super) device: String,
    /// Whether the render contains per-user content.
    pub(super) authenticated: bool,
}

/// Read a parts map.
///
/// # Arguments
///
/// * `value` — The parts map the script built.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The validated [`Parts`].
///
/// # Errors
///
/// Returns an error when `value` is not a map, when `slug` is missing or fails
/// slug validation, when an optional field is present but not a str, when a
/// token field falls outside `[a-z0-9_-]`, or when `authenticated` is present and
/// not a bool.
pub(super) fn read(value: &Value, label: &str) -> Result<Parts, String> {
    let entries = map_arg(value, &format!("{label}: parts"))?;
    let slug = match entries.get("slug") {
        None | Some(Value::Nil) => return Err(format!("{label}: parts is missing `slug`")),
        Some(found) => dynpage_slug::parse(&str_arg(found, &format!("{label}: `slug`"))?, label)?,
    };
    Ok(Parts {
        slug,
        locale: token(&entries, "locale", label)?,
        variant: token(&entries, "variant", label)?,
        device: token(&entries, "device", label)?,
        authenticated: flag(&entries, label)?,
    })
}

/// Read an optional token field, restricted to the slug charset.
///
/// The same charset is reused deliberately: it excludes every control byte, which
/// is what lets [`super::dynpage_key`] use one as its separator.
fn token(entries: &HashMap<String, Value>, key: &str, label: &str) -> Result<String, String> {
    let Some(found) = entries.get(key) else {
        return Ok(String::new());
    };
    if matches!(found, Value::Nil) {
        return Ok(String::new());
    }
    let text = str_arg(found, &format!("{label}: `{key}`"))?.to_ascii_lowercase();
    if !text.is_empty() && !dynpage_slug::valid(&text) {
        return Err(format!("{label}: `{key}` `{text}` must match [a-z0-9_-]+"));
    }
    Ok(text)
}

/// Read `authenticated`, defaulting to the cacheable answer when absent.
fn flag(entries: &HashMap<String, Value>, label: &str) -> Result<bool, String> {
    match entries.get("authenticated") {
        None | Some(Value::Nil) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(other) => Err(format!(
            "{label}: `authenticated` must be bool, got {}",
            other.type_name()
        )),
    }
}
