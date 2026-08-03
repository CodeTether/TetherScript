//! Status-map projection of a channel select outcome.

use crate::value::Value;

use super::result;
use super::select::SelectOutcome;

/// Project a select outcome into the script-visible status map.
///
/// # Arguments
///
/// * `outcome` — Result of the receiver scan.
/// * `handles` — Handles in scan order, used to report the winning channel.
///
/// # Returns
///
/// A map whose `status` is `"value"`, `"end"`, or `"parked"`. The status field
/// exists because a channel may legitimately carry `nil`, so `nil` cannot double
/// as end-of-stream.
pub(super) fn describe(outcome: SelectOutcome, handles: &[i64]) -> Value {
    match outcome {
        SelectOutcome::Ready(index, value) => result::map(vec![
            ("status", result::text("value")),
            ("index", Value::Int(index as i64)),
            ("channel", Value::Int(handles[index])),
            ("value", value),
        ]),
        SelectOutcome::Ended(index) => result::map(vec![
            ("status", result::text("end")),
            ("index", Value::Int(index as i64)),
            ("channel", Value::Int(handles[index])),
        ]),
        SelectOutcome::Parked => result::map(vec![("status", result::text("parked"))]),
    }
}
