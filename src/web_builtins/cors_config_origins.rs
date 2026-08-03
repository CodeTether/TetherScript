//! Reading the `origins` field, including the one wildcard spelling.
//!
//! # Security
//!
//! The wildcard has exactly one spelling — the bare string `"*"` in place of the
//! list — so the single dangerous setting is greppable and cannot be reached by
//! accident from a list of real origins. See `cors_config_syntax` for why a list
//! entry may not be `"*"`.
//!
//! The wildcard is checked against `credentials` in `cors_config`, not here,
//! because that conflict is a property of the *pair* of fields.

use super::cors_args::str_arg;
use crate::value::Value;

/// Parse the `origins` config field.
///
/// # Arguments
///
/// * `value` — The `origins` field: a list of exact origins, or the str `"*"`.
///
/// # Returns
///
/// `(wildcard, origins)`. When `wildcard` is true the list is empty, because a
/// wildcard policy compares nothing.
///
/// # Errors
///
/// Returns an error when the field is missing, is neither a list nor `"*"`, is an
/// empty list, or holds an entry that is not a syntactically valid origin.
pub(super) fn parse(value: Option<&Value>) -> Result<(bool, Vec<String>), String> {
    match value {
        None | Some(Value::Nil) => Err(REQUIRED.to_string()),
        Some(Value::Str(text)) if text.as_str() == "*" => Ok((true, Vec::new())),
        Some(Value::Str(text)) => Err(format!(
            "cors_policy: `origins` as a str must be exactly \"*\", got \"{text}\"; \
             wrap a single origin in a list"
        )),
        Some(Value::List(items)) => Ok((false, exact(&items.borrow()[..])?)),
        Some(other) => Err(format!(
            "cors_policy: `origins` must be a list or the str \"*\", got {}",
            other.type_name()
        )),
    }
}

/// The message for a missing `origins` field.
const REQUIRED: &str = "cors_policy: `origins` is required; pass a list of exact origins, \
     or the string \"*\" for a public API with no credentials";

/// Validate every entry of an origin allow-list.
fn exact(items: &[Value]) -> Result<Vec<String>, String> {
    if items.is_empty() {
        return Err(
            "cors_policy: `origins` must not be an empty list; an empty allow-list \
             rejects every origin, which is better spelled by not installing a CORS layer"
                .to_string(),
        );
    }
    let mut origins = Vec::with_capacity(items.len());
    for item in items {
        let origin = str_arg(item, "cors_policy: `origins` entry")?;
        super::cors_config_syntax::check(&origin)?;
        origins.push(origin);
    }
    Ok(origins)
}
