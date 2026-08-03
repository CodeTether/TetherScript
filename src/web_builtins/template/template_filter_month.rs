//! Month names for `date` formatting.
//!
//! A table rather than a match so `%b` and `%B` share one source of truth: two separate
//! matches would let the abbreviated and full lists drift apart.

/// Abbreviated then full month names, indexed from January.
pub(super) const MONTHS: [(&str, &str); 12] = [
    ("Jan", "January"),
    ("Feb", "February"),
    ("Mar", "March"),
    ("Apr", "April"),
    ("May", "May"),
    ("Jun", "June"),
    ("Jul", "July"),
    ("Aug", "August"),
    ("Sep", "September"),
    ("Oct", "October"),
    ("Nov", "November"),
    ("Dec", "December"),
];
