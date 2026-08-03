//! The maps a script sees.
//!
//! Kept separate from the decision logic so the script-visible shapes live in one
//! place and a reader can see the whole surface without following the branches that
//! produce it.
//!
//! # The assignment map
//!
//! | Field | Meaning |
//! |---|---|
//! | `variant` | The variant name to serve |
//! | `source` | `"cookie"` when pinned, `"computed"` when freshly bucketed |
//! | `set_cookie` | bool: whether the caller should emit a `Set-Cookie` |
//! | `cookie_name` | Configured sticky cookie name, or nil |
//! | `bucket` | The hash bucket, or nil when the cookie decided |
//!
//! `set_cookie` is advisory, and **the caller must act on it**: nothing here writes
//! a response. Ignoring it means the visitor is re-bucketed on every request, which
//! is still stable while the weights hold but stops being stable the moment a weight
//! changes — precisely the failure the sticky cookie exists to prevent. Pair it with
//! `cookie_serialize(name, variant, opts)`.
//!
//! `bucket` is nil for a cookie-sourced assignment rather than recomputed, because
//! reporting a bucket that did not determine the variant would be actively
//! misleading when a weight has moved since the cookie was set.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Build the assignment map for a variant pinned by an existing cookie.
///
/// # Arguments
///
/// * `variant` — Variant name read from the cookie.
/// * `cookie` — The sticky cookie name it came from.
///
/// # Returns
///
/// A map with `source` `"cookie"` and `set_cookie` false: the cookie the browser
/// already holds is the one that should stay.
pub(super) fn from_cookie(variant: &str, cookie: &str) -> Value {
    assignment(variant, "cookie", false, Some(cookie), Value::Nil)
}

/// Build the assignment map for a freshly computed variant.
///
/// # Arguments
///
/// * `variant` — Variant the bucket selected.
/// * `bucket` — The bucket in `0..10000`, reported for observability.
/// * `cookie` — Configured sticky cookie name, if any.
///
/// # Returns
///
/// A map with `source` `"computed"`. `set_cookie` is true exactly when a sticky
/// cookie is configured, since there is nothing to set otherwise.
pub(super) fn computed(variant: &str, bucket: i64, cookie: Option<&str>) -> Value {
    assignment(
        variant,
        "computed",
        cookie.is_some(),
        cookie,
        Value::Int(bucket),
    )
}

/// Assemble the assignment map.
fn assignment(
    variant: &str,
    source: &str,
    set_cookie: bool,
    cookie: Option<&str>,
    bucket: Value,
) -> Value {
    let mut fields: HashMap<String, Value> = HashMap::new();
    fields.insert("variant".into(), text(variant));
    fields.insert("source".into(), text(source));
    fields.insert("set_cookie".into(), Value::Bool(set_cookie));
    fields.insert("cookie_name".into(), cookie.map_or(Value::Nil, text));
    fields.insert("bucket".into(), bucket);
    Value::Map(Rc::new(RefCell::new(fields)))
}

/// Wrap a `&str` as a script string value.
fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}
