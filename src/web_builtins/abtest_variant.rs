//! Variant list parsing.
//!
//! # The weight convention, stated once
//!
//! A weight is an **integer number of percentage points**, and the weights of an
//! experiment must sum to **exactly 100**. A configuration that violates that is
//! rejected by `ab_experiment`, and the error names the sum it actually saw, so the
//! mistake is caught once at construction instead of quietly skewing traffic for
//! the entire life of the experiment.
//!
//! Normalising a bad sum was considered and rejected: `40 + 40` normalised to
//! 50/50 looks like it worked, and the operator who meant `40/60` never finds out.
//!
//! A weight of `0` *is* allowed, so a `100/0` split can be deployed to park an
//! experiment without deleting its variants. A zero-weight variant occupies an
//! empty half-open bucket range and is therefore never selected.
//!
//! The invariant checks themselves live in `abtest_weight`.

use super::abtest_args as args;
use super::abtest_weight as weight;
use crate::value::Value;

/// One configured variant.
pub(super) struct Variant {
    /// Variant name as the script spelled it. Returned verbatim by an assignment.
    pub(super) name: String,
    /// Share of traffic in percentage points.
    pub(super) weight: i64,
}

/// Parse and validate the `variants` list of an experiment config.
///
/// # Arguments
///
/// * `value` — The `variants` field: a list of `{name, weight}` maps.
/// * `label` — Built-in name used in error messages.
///
/// # Returns
///
/// The variants in configured order. Order is load-bearing: it fixes which bucket
/// range each variant owns, so reordering the list reshuffles assignments even when
/// the weights are unchanged.
///
/// # Errors
///
/// Returns an error when `value` is not a list, when the list is empty, when an
/// entry is not a map, when `name` is missing or empty, when `weight` is missing or
/// not an integer, when a weight is negative, when two variants share a name, or
/// when the weights do not sum to exactly 100.
pub(super) fn parse(value: &Value, label: &str) -> Result<Vec<Variant>, String> {
    let Value::List(items) = value else {
        return Err(format!(
            "{label}: `variants` must be a list, got {}",
            value.type_name()
        ));
    };
    let entries = items.borrow();
    if entries.is_empty() {
        return Err(format!(
            "{label}: `variants` must not be empty; an experiment with no variants has nothing to assign"
        ));
    }
    let variants = entries
        .iter()
        .enumerate()
        .map(|(index, item)| one(item, index, label))
        .collect::<Result<Vec<Variant>, String>>()?;
    weight::check_unique(&variants, label)?;
    weight::check_total(&variants, label)?;
    Ok(variants)
}

/// Read a single `{name, weight}` entry.
fn one(item: &Value, index: usize, label: &str) -> Result<Variant, String> {
    let at = format!("{label}: variant {index}");
    let entry = args::map_arg(item, &at)?;
    let name = args::field_str(&entry, "name", &at)?;
    let share = weight::read(&entry, &at, &name)?;
    Ok(Variant { name, weight: share })
}