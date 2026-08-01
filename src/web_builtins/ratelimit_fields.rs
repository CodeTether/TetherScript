//! Reading numeric fields out of a caller-supplied bucket map.
//!
//! Every failure names the field, because a bucket arrives from script code and a
//! silent default would turn a typo into either an unlimited or a permanently
//! closed limiter.

use std::collections::HashMap;

use crate::value::Value;

/// Read a required numeric field, accepting int or float.
///
/// # Arguments
///
/// * `bucket` — The caller's bucket map.
/// * `field` — Field name to read.
///
/// # Returns
///
/// The value as `f64`.
///
/// # Errors
///
/// Returns an error naming `field` when it is absent or not numeric.
pub(super) fn number(bucket: &HashMap<String, Value>, field: &str) -> Result<f64, String> {
    match bucket.get(field) {
        Some(Value::Int(int)) => Ok(*int as f64),
        Some(Value::Float(float)) => Ok(*float),
        Some(other) => Err(format!(
            "bucket field `{field}` must be a number, got {}",
            other.type_name()
        )),
        None => Err(format!("bucket is missing the `{field}` field")),
    }
}

/// Read a required integer field, such as the last-refill timestamp.
///
/// # Errors
///
/// Returns an error naming `field` when it is absent or not an int.
pub(super) fn integer(bucket: &HashMap<String, Value>, field: &str) -> Result<i64, String> {
    match bucket.get(field) {
        Some(Value::Int(int)) => Ok(*int),
        Some(other) => Err(format!(
            "bucket field `{field}` must be an int, got {}",
            other.type_name()
        )),
        None => Err(format!("bucket is missing the `{field}` field")),
    }
}

/// Coerce a built-in argument to a number, naming the parameter on mismatch.
pub(super) fn num_arg(value: &Value, label: &str) -> Result<f64, String> {
    match value {
        Value::Int(int) => Ok(*int as f64),
        Value::Float(float) => Ok(*float),
        other => Err(format!(
            "{label} must be a number, got {}",
            other.type_name()
        )),
    }
}

/// Require a strictly positive value.
///
/// # Errors
///
/// Returns an error naming `label` when `value` is zero or negative.
pub(super) fn positive(value: f64, label: &str) -> Result<(), String> {
    if value <= 0.0 {
        return Err(format!("{label} must be > 0, got {value}"));
    }
    Ok(())
}

/// Validate a take cost against the bucket capacity.
///
/// # Errors
///
/// Returns an error when `cost` is not positive, or when it exceeds `capacity`.
/// The second case can never succeed, so treating it as an ordinary denial would
/// leave the caller retrying forever.
pub(super) fn check_cost(cost: f64, capacity: f64) -> Result<(), String> {
    positive(cost, "bucket_take: cost")?;
    if cost > capacity {
        return Err(format!(
            "bucket_take: cost {cost} exceeds capacity {capacity}; this can never succeed"
        ));
    }
    Ok(())
}
