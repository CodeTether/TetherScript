//! Request-driven assignment. An existing sticky cookie wins.
//!
//! # Why the cookie wins over a fresh computation
//!
//! If the recomputed value were preferred, then changing a variant's weight
//! mid-experiment would migrate visitors who had already been exposed: someone who
//! saw the control for a week would suddenly see the treatment, and their behaviour
//! would be attributed to a variant they had barely encountered. Honouring the
//! cookie pins a visitor for the life of their cookie, so a weight change only ever
//! affects subjects that had not yet been bucketed.
//!
//! A cookie naming a variant that is no longer configured is *not* honoured — it is
//! discarded and the subject is re-bucketed, because reporting a variant that does
//! not exist would break analysis just as badly.
//!
//! # Where the subject comes from
//!
//! From the request map's `subject` field: the caller decides what identifies a
//! visitor (a visitor cookie, a session id, an account id) and puts it there. It is
//! required only when no usable cookie is present, so a returning visitor is served
//! without the handler having to resolve an identity at all.

use std::collections::HashMap;

use super::abtest_args as args;
use super::abtest_assign as assign;
use super::abtest_bucket as hash;
use super::abtest_config::Experiment;
use super::abtest_cookie as jar;
use super::abtest_shape as shape;
use crate::value::Value;

/// Decide the variant for a request.
///
/// # Arguments
///
/// * `experiment` — The validated experiment.
/// * `request` — The request map: `headers`, optional parsed `cookies`, and
///   `subject`.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The assignment map `abtest_shape` documents: `variant`, `source`, `set_cookie`,
/// `cookie_name`, and `bucket`.
///
/// # Errors
///
/// Returns an error when `request` is not a map, when `cookies` or `headers` is
/// present but not a map, or when no usable sticky cookie is present and `subject`
/// is missing, empty, or not a str.
pub(super) fn decide(
    experiment: &Experiment,
    request: &Value,
    label: &str,
) -> Result<Value, String> {
    let entries = args::map_arg(request, &format!("{label}: request"))?;
    if let Some(name) = &experiment.sticky_cookie {
        if let Some(existing) = honoured(experiment, &entries, name, label)? {
            return Ok(shape::from_cookie(&existing, name));
        }
    }
    compute(experiment, &entries, label)
}

/// Read the sticky cookie, keeping it only if it names a configured variant.
fn honoured(
    experiment: &Experiment,
    entries: &HashMap<String, Value>,
    cookie: &str,
    label: &str,
) -> Result<Option<String>, String> {
    let Some(value) = jar::read(entries, cookie, label)? else {
        return Ok(None);
    };
    Ok(experiment
        .variants
        .iter()
        .find(|variant| variant.name == value)
        .map(|variant| variant.name.clone()))
}

/// Bucket the subject and report that a cookie should be set.
fn compute(
    experiment: &Experiment,
    entries: &HashMap<String, Value>,
    label: &str,
) -> Result<Value, String> {
    let subject = args::field_str(entries, "subject", label)?;
    let bucket = hash::bucket(&experiment.seed, &subject);
    let variant = assign::select(&experiment.variants, bucket);
    Ok(shape::computed(
        &variant,
        bucket,
        experiment.sticky_cookie.as_deref(),
    ))
}
