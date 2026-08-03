//! Handle-list coercion for the channel select built-in.

use crate::value::Value;

use super::args;

/// Read a list-of-handles argument.
///
/// # Arguments
///
/// * `value` — Candidate list value.
///
/// # Returns
///
/// The handles in list order, which is also the select priority order.
///
/// # Errors
///
/// Returns `Err` naming the offending type when the value is not a list, or when
/// any element is not an int handle.
pub(super) fn handle_list(value: &Value) -> Result<Vec<i64>, String> {
    let Value::List(list) = value else {
        return Err(format!(
            "chan_select: expected a list of channel handles, got {}",
            value.type_name()
        ));
    };
    list.borrow()
        .iter()
        .map(|entry| args::handle(entry, "chan_select"))
        .collect()
}
