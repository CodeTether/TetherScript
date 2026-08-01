//! Map assembly for bucket state and take results.
//!
//! Separated from the decision logic so the shapes the script sees live in one
//! place, matching how `response.tether` keeps response shaping separate.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::ratelimit_bucket as bucket;
use crate::value::Value;

/// Assemble a bucket map.
///
/// Tokens are stored as a float because a partial token is meaningful: at 2/sec a
/// 300ms gap earns 0.6 of a token, and truncating that to an int would discard
/// most of a slow client's refill.
pub(super) fn bucket_map(capacity: f64, tokens: f64, refill_per_sec: f64, updated: i64) -> Value {
    let mut state = HashMap::new();
    state.insert(bucket::CAPACITY.into(), Value::Float(capacity));
    state.insert(bucket::TOKENS.into(), Value::Float(tokens));
    state.insert(bucket::REFILL.into(), Value::Float(refill_per_sec));
    state.insert(bucket::UPDATED.into(), Value::Int(updated));
    Value::Map(Rc::new(RefCell::new(state)))
}

/// Assemble the take result: the decision, the next bucket, and the wait.
pub(super) fn outcome(allowed: bool, next: Value, retry_after_ms: i64) -> Value {
    let mut result = HashMap::new();
    result.insert("allowed".into(), Value::Bool(allowed));
    result.insert("bucket".into(), next);
    result.insert("retry_after_ms".into(), Value::Int(retry_after_ms));
    Value::Map(Rc::new(RefCell::new(result)))
}
