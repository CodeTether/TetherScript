//! Adapters from script `Value`s into the A/B logic.
//!
//! Each function here is the whole body of one built-in: coerce, delegate, wrap.
//! Registration lives in `abtest_install`, the decisions live in the modules these
//! call, and the canonical experiment map is rebuilt by `abtest_echo`, so this file
//! holds no logic of its own.

use super::abtest_args as args;
use super::abtest_assign as assign;
use super::abtest_bucket as hash;
use super::abtest_config as config;
use super::abtest_echo as echo;
use super::abtest_request as request;
use crate::value::Value;

/// `ab_experiment(config)` — validate a config and echo it back canonically.
///
/// # Arguments
///
/// * `value` — The config map: `name`, `seed`, `variants`, optional `sticky_cookie`.
///
/// # Returns
///
/// A freshly built experiment map. See `abtest_echo` for why it is rebuilt rather
/// than returned by reference.
///
/// # Errors
///
/// Every rejection from [`config::read`]: a non-map config, a missing or empty
/// `name` or `seed`, zero variants, a negative weight, a duplicate variant name, or
/// weights not summing to 100.
pub(super) fn experiment(value: &Value) -> Result<Value, String> {
    Ok(echo::experiment_map(&config::read(value, "ab_experiment")?))
}

/// `ab_assign(experiment, subject)` — the variant name for a stable subject.
///
/// # Arguments
///
/// * `experiment` — An experiment map, revalidated on every call.
/// * `subject` — Stable subject identifier: a visitor id or session id.
///
/// # Returns
///
/// The variant name. A pure function of the experiment's seed and the subject: the
/// same pair yields the same name in every process, forever.
///
/// # Errors
///
/// Any error from [`config::read`], or a `subject` that is not a non-empty str.
pub(super) fn assign(experiment: &Value, subject: &Value) -> Result<Value, String> {
    let parsed = config::read(experiment, "ab_assign")?;
    let subject = args::nonempty_str(subject, "ab_assign: subject")?;
    let bucket = hash::bucket(&parsed.seed, &subject);
    Ok(echo::str_value(&assign::select(&parsed.variants, bucket)))
}

/// `ab_assign_from_request(experiment, request)` — assignment plus cookie advice.
///
/// # Arguments
///
/// * `experiment` — An experiment map.
/// * `req` — The request map: `headers`, optional parsed `cookies`, and `subject`.
///
/// # Returns
///
/// The assignment map documented in `abtest_shape`. An existing sticky cookie wins
/// over a fresh computation.
///
/// # Errors
///
/// Any error from [`config::read`] or [`request::decide`].
pub(super) fn from_request(experiment: &Value, req: &Value) -> Result<Value, String> {
    let label = "ab_assign_from_request";
    let parsed = config::read(experiment, label)?;
    request::decide(&parsed, req, label)
}

/// `ab_bucket(seed, subject)` — the raw hash bucket in `0..10000`.
///
/// Exposed so a test, or a script auditing its own traffic split, can observe the
/// distribution directly instead of inferring it from variant counts.
///
/// # Arguments
///
/// * `seed` — Experiment seed.
/// * `subject` — Stable subject identifier.
///
/// # Returns
///
/// An int in `0..10000`.
///
/// # Errors
///
/// Returns an error when either argument is not a non-empty str.
pub(super) fn bucket(seed: &Value, subject: &Value) -> Result<Value, String> {
    let seed = args::nonempty_str(seed, "ab_bucket: seed")?;
    let subject = args::nonempty_str(subject, "ab_bucket: subject")?;
    Ok(Value::Int(hash::bucket(&seed, &subject)))
}