//! Assembly of the policy map a validated config produces.
//!
//! Kept apart from validation so the shape the script sees lives in one place,
//! and so the reader in `cors_policy_read` has a single definition to match.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::cors_fields as key;
use crate::value::Value;

/// The validated fields a policy map is assembled from.
pub(super) struct Fields {
    /// Whether `origins` was the wildcard `"*"`.
    pub(super) wildcard: bool,
    /// Exact origins; empty when `wildcard`.
    pub(super) origins: Vec<String>,
    /// Allowed methods, upper-cased.
    pub(super) methods: Vec<String>,
    /// Allowed request header names, lower-cased.
    pub(super) headers: Vec<String>,
    /// Exposed response header names, lower-cased.
    pub(super) expose: Vec<String>,
    /// Whether cookies and `Authorization` may accompany the request.
    pub(super) credentials: bool,
    /// Preflight cache lifetime in seconds, when requested.
    pub(super) max_age: Option<i64>,
}

/// Wrap a token list as a script list value.
fn tokens(items: Vec<String>) -> Value {
    let values = items.into_iter().map(|t| Value::Str(Rc::new(t))).collect();
    Value::List(Rc::new(RefCell::new(values)))
}

/// Build the policy map a script holds onto.
///
/// # Arguments
///
/// * `fields` — The validated fields.
///
/// # Returns
///
/// A map with `wildcard`, `origins`, `methods`, `headers`, `expose`,
/// `credentials`, and, when set, `max_age`.
pub(super) fn policy(fields: Fields) -> Value {
    let mut map = HashMap::new();
    map.insert(key::WILDCARD.into(), Value::Bool(fields.wildcard));
    map.insert(key::ORIGINS.into(), tokens(fields.origins));
    map.insert(key::METHODS.into(), tokens(fields.methods));
    map.insert(key::HEADERS.into(), tokens(fields.headers));
    map.insert(key::EXPOSE.into(), tokens(fields.expose));
    map.insert(key::CREDENTIALS.into(), Value::Bool(fields.credentials));
    if let Some(seconds) = fields.max_age {
        map.insert(key::MAX_AGE.into(), Value::Int(seconds));
    }
    Value::Map(Rc::new(RefCell::new(map)))
}
