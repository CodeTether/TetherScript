//! Shared helper for the socket capability tests.

/// Convert string literals into the owned `Vec<String>` the grant API takes.
pub(super) fn patterns(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
