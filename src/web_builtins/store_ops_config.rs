//! `store_configure` and `store_count`: policy and observability.
//!
//! Configuration applies to sessions created *after* the call. Existing records
//! carry the TTLs they were created with, so shortening the policy cannot be used
//! to retroactively kill live sessions — use `store_destroy_subject` for that,
//! where the intent is explicit — and lengthening it cannot silently resurrect the
//! ceiling on a session an operator already believes is bounded.

use super::store_args::int_arg;
// The trait must be in scope to call `count` through the boxed backend.
// `SessionBackend` is reached through the concrete types below rather than the trait.
use super::store_state;
use crate::value::Value;

/// `store_configure(idle_ttl_ms, absolute_ttl_ms)` — set the policy for new
/// sessions.
///
/// # Arguments
///
/// * `args` — `[idle_ttl_ms: int, absolute_ttl_ms: int]`. `0` disables that clock.
///
/// # Returns
///
/// Nil.
///
/// # Errors
///
/// Returns a named error when either argument is not an int or is negative.
///
/// A ceiling shorter than the idle window is deliberately **allowed**: "a hard
/// five-minute session regardless of activity" is a legitimate policy, and it is
/// expressed exactly that way. In that configuration the ceiling simply always
/// fires first, which is what the operator asked for.
pub(super) fn configure(args: &[Value]) -> Result<Value, String> {
    let idle = int_arg(&args[0], "store_configure: idle_ttl_ms")?;
    let absolute = int_arg(&args[1], "store_configure: absolute_ttl_ms")?;
    for (label, value) in [("idle_ttl_ms", idle), ("absolute_ttl_ms", absolute)] {
        if value < 0 {
            return Err(format!(
                "store_configure: {label} must not be negative, got {value}"
            ));
        }
    }
    store_state::with(|store| {
        store.idle_ttl_ms = idle;
        store.absolute_ttl_ms = absolute;
    });
    Ok(Value::Nil)
}

/// `store_count()` — how many records the backend holds.
///
/// # Returns
///
/// The count, including expired-but-unswept records. Intended for tests and
/// operational metrics, not for authorisation decisions.
///
/// # Errors
///
/// Returns an error only when the backend fails.
pub(super) fn count() -> Result<Value, String> {
    store_state::with(|store| Ok(Value::Int(store.backend.count()? as i64)))
}
