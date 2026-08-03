//! # Human-readable rendering of [`DecodeError`]
//!
//! Split from the type itself so the error vocabulary and its wording stay
//! independently reviewable. Every message names the offending field, because
//! "decode error" tells an operator nothing about which column to cast.
//!
//! All messages are prefixed `postgres:` so they read correctly when the
//! integrator converts them with `.map_err(|error| error.to_string())` into the
//! `Result<_, String>` the rest of `src/postgres/` uses.

use std::fmt;

use super::DecodeError;

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Truncated { what, need, have } => write!(
                f,
                "postgres: truncated binary {what}: needs {need} byte(s), {have} available"
            ),
            DecodeError::Overlong {
                what,
                expected,
                got,
            } => write!(
                f,
                "postgres: over-long binary {what}: layout consumes {expected} byte(s) but {got} were sent"
            ),
            DecodeError::UnsupportedOid { oid } => write!(
                f,
                "postgres: no binary decoder for type OID {oid}; decode this column as text"
            ),
            DecodeError::UnsupportedDimensions { ndim } => write!(
                f,
                "postgres: binary array has {ndim} dimension(s); only 0 or 1 are supported"
            ),
            DecodeError::BadUtf8 { what } => {
                write!(f, "postgres: binary {what} field is not valid UTF-8")
            }
            DecodeError::BadNumericSign { sign } => write!(
                f,
                "postgres: binary numeric has unrecognised sign word 0x{sign:04X}"
            ),
            DecodeError::BadValue { what, detail } => {
                write!(f, "postgres: invalid binary {what}: {detail}")
            }
        }
    }
}
