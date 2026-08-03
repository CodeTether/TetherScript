//! Rebuilding the canonical experiment map a script receives.
//!
//! `ab_experiment` does not hand back the caller's own map. It builds a new one, so
//! a script that mutates its original config afterwards cannot retroactively
//! invalidate an experiment already in use — the validation `ab_experiment` performed
//! would otherwise be a claim about a value that has since changed.
//!
//! The rebuilt map is deliberately the same shape `abtest_config` reads, so an
//! experiment round-trips: `ab_assign(ab_experiment(cfg)?, s)` and
//! `ab_assign(cfg, s)` agree. That is also why every assignment revalidates rather
//! than trusting the map it is given.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::abtest_config::{self as config, Experiment};
use crate::value::Value;

/// Build the script-visible experiment map from a validated experiment.
///
/// # Arguments
///
/// * `parsed` — The validated experiment.
///
/// # Returns
///
/// A map with `name`, `seed`, `variants` (in configured order), and
/// `sticky_cookie`, which is nil when none is configured rather than absent, so a
/// script can read the field unconditionally.
pub(super) fn experiment_map(parsed: &Experiment) -> Value {
    let mut fields: HashMap<String, Value> = HashMap::new();
    fields.insert(config::NAME.into(), str_value(&parsed.name));
    fields.insert(config::SEED.into(), str_value(&parsed.seed));
    fields.insert(config::VARIANTS.into(), variant_list(parsed));
    fields.insert(
        config::STICKY_COOKIE.into(),
        parsed.sticky_cookie.as_deref().map_or(Value::Nil, str_value),
    );
    Value::Map(Rc::new(RefCell::new(fields)))
}

/// Rebuild the variant list, preserving configured order.
///
/// Order is preserved rather than sorted because it determines which bucket range
/// each variant owns; sorting would silently reshuffle every assignment.
fn variant_list(parsed: &Experiment) -> Value {
    let items = parsed
        .variants
        .iter()
        .map(|variant| {
            let mut entry: HashMap<String, Value> = HashMap::new();
            entry.insert("name".into(), str_value(&variant.name));
            entry.insert("weight".into(), Value::Int(variant.weight));
            Value::Map(Rc::new(RefCell::new(entry)))
        })
        .collect();
    Value::List(Rc::new(RefCell::new(items)))
}

/// Wrap a `&str` as a script string value.
///
/// # Arguments
///
/// * `value` — Text to wrap.
///
/// # Returns
///
/// The equivalent [`Value::Str`].
pub(super) fn str_value(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}
