//! The weight invariants: field type, non-negativity, uniqueness, and the sum.
//!
//! Split from `abtest_variant` so that module handles only the shape of the list
//! while this one owns the rules the list must satisfy. Every rejection here names
//! the offending variant and the value observed, because a skewed experiment is
//! otherwise invisible until the results are already worthless.

use std::collections::{HashMap, HashSet};

use super::abtest_variant::Variant;
use crate::value::Value;

/// Required sum of all variant weights, in percentage points.
pub(super) const TOTAL_WEIGHT: i64 = 100;

/// Read the `weight` field of a variant entry as a non-negative integer.
///
/// # Arguments
///
/// * `entry` — The variant map.
/// * `at` — Position prefix for errors, e.g. `"ab_experiment: variant 1"`.
/// * `name` — Variant name, so the error identifies the variant by name and index.
///
/// # Returns
///
/// The weight in percentage points, `>= 0`.
///
/// # Errors
///
/// Returns an error when `weight` is absent or nil, when it is not an int (a float
/// weight is refused rather than truncated, because `33.3` truncated to `33` would
/// break the sum check in a way the operator did not write), or when it is negative.
pub(super) fn read(entry: &HashMap<String, Value>, at: &str, name: &str) -> Result<i64, String> {
    let weight = match entry.get("weight") {
        None | Some(Value::Nil) => return Err(format!("{at} `{name}`: missing `weight`")),
        Some(Value::Int(weight)) => *weight,
        Some(other) => {
            return Err(format!(
                "{at} `{name}`: `weight` must be int percentage points, got {}",
                other.type_name()
            ));
        }
    };
    if weight < 0 {
        return Err(format!(
            "{at} `{name}`: `weight` must not be negative, got {weight}"
        ));
    }
    Ok(weight)
}

/// Reject two variants sharing a name.
///
/// Duplicates are ambiguous rather than merely redundant: an assignment returns a
/// name, so two variants called `control` would report results that cannot be told
/// apart, and the experiment could not be analysed at all.
///
/// # Errors
///
/// Returns an error naming the repeated variant.
pub(super) fn check_unique(variants: &[Variant], label: &str) -> Result<(), String> {
    let mut seen: HashSet<&str> = HashSet::new();
    for variant in variants {
        if !seen.insert(variant.name.as_str()) {
            return Err(format!(
                "{label}: duplicate variant name `{}`; variant names must be unique",
                variant.name
            ));
        }
    }
    Ok(())
}

/// Reject weights that do not sum to exactly [`TOTAL_WEIGHT`].
///
/// # Errors
///
/// Returns an error stating both the required total and the sum observed, so the
/// operator can see the size of the mistake without recomputing it.
pub(super) fn check_total(variants: &[Variant], label: &str) -> Result<(), String> {
    let sum: i64 = variants.iter().map(|variant| variant.weight).sum();
    if sum != TOTAL_WEIGHT {
        return Err(format!(
            "{label}: variant weights must sum to {TOTAL_WEIGHT}, got {sum}"
        ));
    }
    Ok(())
}
