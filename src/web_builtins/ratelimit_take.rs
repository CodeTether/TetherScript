//! Bucket construction and the take decision.

use std::collections::HashMap;

use super::ratelimit_bucket as bucket;
use super::ratelimit_fields as fields;
use super::ratelimit_shape as shape;
use crate::value::Value;

/// Build a fresh, full bucket.
///
/// # Arguments
///
/// * `capacity` — Maximum tokens, which is also the largest allowed burst.
/// * `refill_per_sec` — Tokens restored per second.
///
/// # Returns
///
/// A bucket map starting full, stamped with the current time.
///
/// # Errors
///
/// Returns an error when either argument is not strictly positive. A zero
/// capacity would deny every request forever and a zero refill would never
/// recover, so neither is accepted silently.
pub(super) fn new(capacity: f64, refill_per_sec: f64) -> Result<Value, String> {
    fields::positive(capacity, "bucket_new: capacity")?;
    fields::positive(refill_per_sec, "bucket_new: refill_per_sec")?;
    Ok(shape::bucket_map(
        capacity,
        capacity,
        refill_per_sec,
        bucket::now_ms(),
    ))
}

/// Attempt to spend `cost` tokens, returning the outcome and the next bucket.
///
/// # Arguments
///
/// * `state` — The caller's current bucket map.
/// * `cost` — Tokens this request consumes.
///
/// # Returns
///
/// A map with `allowed`, the updated `bucket`, and `retry_after_ms` (0 when
/// allowed). **The caller must persist the returned `bucket`**; the input is never
/// mutated, which is what keeps this free of hidden shared state.
///
/// # Errors
///
/// Returns an error when the bucket is malformed, when `cost` is not positive, or
/// when `cost` exceeds capacity. That last case can never succeed, so reporting it
/// as an ordinary denial would leave a caller retrying forever.
pub(super) fn take(state: &HashMap<String, Value>, cost: f64) -> Result<Value, String> {
    let capacity = fields::number(state, bucket::CAPACITY)?;
    let refill = fields::number(state, bucket::REFILL)?;
    let held = fields::number(state, bucket::TOKENS)?;
    let updated = fields::integer(state, bucket::UPDATED)?;
    fields::check_cost(cost, capacity)?;

    let now = bucket::now_ms();
    let available = bucket::refilled(held, capacity, refill, updated, now);
    let allowed = available >= cost;
    let remaining = if allowed { available - cost } else { available };
    let retry_after = if allowed {
        0
    } else {
        bucket::wait_ms(cost - available, refill)
    };

    Ok(shape::outcome(
        allowed,
        shape::bucket_map(capacity, remaining, refill, now),
        retry_after,
    ))
}
