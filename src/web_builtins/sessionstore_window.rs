//! Fixed-window arithmetic: validation, window end, and the retry delay.
//!
//! # The honest limit is 2x, not 1x
//!
//! A fixed window counts requests inside `[index * window, (index + 1) * window)`
//! and resets to zero at the boundary. A client that spends its whole allowance in
//! the last instant of one window and its whole allowance in the first instant of
//! the next issues **2 x limit requests in just over a second of wall clock**, while
//! never exceeding the configured limit in either window. That is inherent to the
//! algorithm, not a defect here, and it is stated plainly because a caller sizing a
//! limit against a fragile upstream needs the real worst case: to cap true burst at
//! N, configure `limit = N / 2`.
//!
//! A sliding window removes the doubling but costs more state: either a sorted set
//! of per-request timestamps per subject — O(requests) memory and a trim on every
//! call — or two counters plus weighted interpolation of the previous window, which
//! is approximate at a fraction of the cost. Fixed windows need one integer per
//! subject per window and a single atomic `INCR`, which is why they are the default.
//!
//! # `reset_at` is computed, never stored
//!
//! `reset_at` is the end of the window containing `now`, derived from `now` and the
//! window size. A stored expiry could drift from the key's own boundary after a
//! restart or a re-`EXPIRE`, and `Retry-After` must be truthful: too early invites a
//! rejected retry, too late wastes the client's allowance.

/// Reject a non-positive window.
///
/// # Arguments
///
/// * `label` — Built-in name, used verbatim in the error.
/// * `window_secs` — Candidate window width in seconds.
///
/// # Returns
///
/// `Ok(())` when `window_secs` is at least 1.
///
/// # Errors
///
/// Returns a named error for zero or negative widths. Zero would divide by zero when
/// computing the window index; a negative width has no meaning and would truncate
/// toward zero, aliasing distinct instants onto one bucket.
///
/// # Examples
///
/// ```rust,ignore
/// assert!(require_window("l", 60).is_ok());
/// assert!(require_window("l", 0).is_err());
/// ```
pub(super) fn require_window(label: &str, window_secs: i64) -> Result<(), String> {
    if window_secs <= 0 {
        return Err(format!(
            "{label}: window_secs must be a positive number of seconds, got {window_secs}"
        ));
    }
    Ok(())
}
