//! Array conversion for decoded RESP replies.
//!
//! Split from [`super::handler_value`] so each file owns one concern and stays within the
//! line budget.
//!
//! The kept codec is RESP2, so there are no RESP3 aggregate types to convert here. If
//! RESP3 support lands later, its map and set types belong in this file, keyed the way a
//! script expects rather than as a flat alternating list.

use std::cell::RefCell;
use std::rc::Rc;

use super::value::RespValue;
use crate::value::Value;

/// Convert an array reply to a script list.
///
/// # Arguments
///
/// * `items` — Decoded elements, which may themselves be arrays.
///
/// # Returns
///
/// A [`Value::List`] whose elements are converted recursively, so a nested array becomes
/// a nested list rather than being flattened.
///
/// # Errors
///
/// Returns the first element conversion error, so one bad payload names itself instead of
/// silently dropping out of the list.
pub(super) fn list(items: Vec<RespValue>) -> Result<Value, String> {
    let mut converted = Vec::with_capacity(items.len());
    for item in items {
        converted.push(super::handler_value::from_resp(item)?);
    }
    Ok(Value::List(Rc::new(RefCell::new(converted))))
}
