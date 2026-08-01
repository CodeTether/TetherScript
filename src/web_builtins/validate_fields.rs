//! The `validate_fields` map walk.
//!
//! Owns which fields get checked and in what order; [`super::validate_rule`] owns
//! what each rule means.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::validate_rule::apply;

/// Validate a map of submitted values against a map of rules.
///
/// # Arguments
///
/// * `values` — Submitted field values, typically from `form_parse` or `json_parse`.
/// * `rules` — Map of field name to a rules map, for example `{"email": {"required": true, "email": true}}`.
///
/// # Returns
///
/// A map of field name to the FIRST failing message for that field. The map is
/// empty when everything passes, so a handler can branch on `len() == 0`.
///
/// # Errors
///
/// Returns an error when either argument is not a map, when a field's rule set is
/// not a map, or when a rule is unknown or misconfigured.
pub(super) fn validate_fields(values: &Value, rules: &Value) -> Result<Value, String> {
    let values = as_map(values, "values")?;
    let rules = as_map(rules, "rules")?;
    let values = values.borrow();
    let mut failures: HashMap<String, Value> = HashMap::new();

    for (field, spec) in rules.borrow().iter() {
        let spec = as_map(spec, field)?;
        // Sort so a field with several broken rules reports deterministically
        // rather than depending on map iteration order.
        let mut ordered: Vec<String> = spec.borrow().keys().cloned().collect();
        ordered.sort();

        for rule in ordered {
            let argument = spec.borrow().get(&rule).cloned().unwrap_or(Value::Nil);
            if let Some(message) = apply(field, &rule, &argument, values.get(field))? {
                failures.insert(field.clone(), Value::Str(Rc::new(message)));
                break;
            }
        }
    }

    Ok(Value::Map(Rc::new(RefCell::new(failures))))
}

/// Require a map argument, naming the offending parameter.
fn as_map(value: &Value, label: &str) -> Result<Rc<RefCell<HashMap<String, Value>>>, String> {
    match value {
        Value::Map(map) => Ok(map.clone()),
        other => Err(format!(
            "validate_fields: {label} must be map, got {}",
            other.type_name()
        )),
    }
}
