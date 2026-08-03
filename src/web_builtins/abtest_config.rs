//! The experiment shape: validated once by `ab_experiment`, read back on every use.
//!
//! An experiment is an ordinary tetherscript map, so a script owns it and can store
//! it in a module-level binding, serialise it to JSON, or rebuild it per request.
//! Nothing here holds global mutable state.
//!
//! The map a script receives is intentionally the *same* shape this module reads
//! back, so an assignment re-validates the config rather than trusting it. That
//! matters because a script can hand-build a map and skip `ab_experiment` entirely;
//! re-reading means the weight invariants still hold at assignment time.

use std::collections::HashMap;

use super::abtest_args as args;
use super::abtest_variant::{self as variant, Variant};
use crate::value::Value;

/// Field names on the experiment map, in one place so the reader and the writer
/// cannot disagree about a spelling.
pub(super) const NAME: &str = "name";
pub(super) const SEED: &str = "seed";
pub(super) const VARIANTS: &str = "variants";
pub(super) const STICKY_COOKIE: &str = "sticky_cookie";

/// A validated experiment.
pub(super) struct Experiment {
    /// Human-readable experiment name, used only for error messages and reporting.
    pub(super) name: String,
    /// Hash seed. Changing it reshuffles every subject's assignment.
    pub(super) seed: String,
    /// Variants in configured order, weights summing to 100.
    pub(super) variants: Vec<Variant>,
    /// Cookie name that pins an existing assignment, when one is configured.
    pub(super) sticky_cookie: Option<String>,
}

/// Read and validate an experiment config map.
///
/// # Arguments
///
/// * `value` — The config map: `name`, `seed`, `variants`, optional `sticky_cookie`.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The validated [`Experiment`].
///
/// # Errors
///
/// Returns an error when `value` is not a map, when `name` or `seed` is missing or
/// empty, when `sticky_cookie` is present but not a str, or for any variant
/// violation reported by [`variant::parse`] — zero variants, a negative weight, a
/// duplicate name, or weights not summing to 100.
pub(super) fn read(value: &Value, label: &str) -> Result<Experiment, String> {
    let entries = args::map_arg(value, &format!("{label}: config"))?;
    Ok(Experiment {
        name: args::field_str(&entries, NAME, label)?,
        seed: args::field_str(&entries, SEED, label)?,
        variants: variants_of(&entries, label)?,
        sticky_cookie: args::field_opt_str(&entries, STICKY_COOKIE, label)?,
    })
}

/// Read the `variants` field, reporting its absence distinctly from its contents.
fn variants_of(entries: &HashMap<String, Value>, label: &str) -> Result<Vec<Variant>, String> {
    match entries.get(VARIANTS) {
        None | Some(Value::Nil) => Err(format!("{label}: missing `{VARIANTS}`")),
        Some(value) => variant::parse(value, label),
    }
}
