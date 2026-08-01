//! Bucket argument borrowing for `bucket_take`.

use std::collections::HashMap;

use super::ratelimit_take as ops;
use crate::value::Value;

/// Clone the bucket map out of the argument before deciding.
///
/// The clone is deliberate: the decision returns a *new* bucket rather than
/// mutating the caller's, so nothing here needs a mutable borrow that could alias
/// a map the script still holds.
///
/// # Errors
///
/// Returns an error naming the actual type when the argument is not a map.
pub(super) fn take(bucket: &Value, cost: f64) -> Result<Value, String> {
    let Value::Map(state) = bucket else {
        return Err(format!(
            "bucket_take: bucket must be a map, got {}",
            bucket.type_name()
        ));
    };
    let borrowed: HashMap<String, Value> = state.borrow().clone();
    ops::take(&borrowed, cost)
}
