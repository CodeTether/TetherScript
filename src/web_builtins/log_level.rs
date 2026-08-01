//! Log level parsing, ordering, and `LOG_LEVEL` filtering.
//!
//! Levels are ordered so a threshold admits itself and everything more severe.
//! An unknown level is always an error rather than a silent default, because a
//! typo like `warning` must not quietly downgrade a call to a level that is
//! filtered out and disappears.

/// Severity ordering: lower is more verbose.
const ORDER: [&str; 5] = ["trace", "debug", "info", "warn", "error"];

/// Default threshold when `LOG_LEVEL` is unset or unparseable.
pub(super) const DEFAULT_LEVEL: &str = "info";

/// Rank a level name, case-insensitively.
///
/// # Arguments
///
/// * `level` — Level name such as `info` or `ERROR`.
///
/// # Returns
///
/// The severity index, where `trace` is 0 and `error` is 4.
///
/// # Errors
///
/// Returns an error naming the rejected level and listing the accepted names.
pub(super) fn rank(level: &str) -> Result<usize, String> {
    let lowered = level.to_ascii_lowercase();
    ORDER
        .iter()
        .position(|known| *known == lowered)
        .ok_or_else(|| {
            format!(
                "log: unknown level `{level}`; expected one of {}",
                ORDER.join(", ")
            )
        })
}

/// Canonical lowercase spelling of a level.
///
/// # Errors
///
/// Returns an error when `level` is not a known level name.
pub(super) fn canonical(level: &str) -> Result<&'static str, String> {
    Ok(ORDER[rank(level)?])
}

/// Decide whether `level` passes the `threshold`.
///
/// # Arguments
///
/// * `level` — Level of the call site.
/// * `threshold` — Configured minimum, normally from `LOG_LEVEL`.
///
/// # Returns
///
/// True when `level` is at least as severe as `threshold`. An unparseable
/// threshold falls back to [`DEFAULT_LEVEL`] rather than dropping every line.
///
/// # Errors
///
/// Returns an error when `level` itself is unknown.
pub(super) fn enabled(level: &str, threshold: &str) -> Result<bool, String> {
    let floor = rank(threshold)
        .unwrap_or_else(|_| rank(DEFAULT_LEVEL).expect("the default level must be a known level"));
    Ok(rank(level)? >= floor)
}
