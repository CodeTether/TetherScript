//! `--grant-redis` URL parsing, from the CLI's point of view.
//!
//! One concern: naming the parser the CLI uses. The implementation lives in
//! [`crate::redis_cap::url`] rather than here for one reason: `main_caps` is declared
//! only in `src/main.rs`, so nothing under `tests/` can reach it, and the URL parser is
//! the part of this grant that most needs direct test coverage — every rejection has to
//! be provable without a Redis server running.
//!
//! `tests/grant_redis_url.rs` therefore tests [`crate::redis_cap::url`] through the
//! library, and this module keeps the CLI-side name pointing at it so a reader
//! following `--grant-redis` from `src/main.rs` lands somewhere that explains the
//! arrangement.
//!
//! Contrast `main_caps::db`, whose `parse_url` is a private sibling tested from
//! `main_caps::db_tests` as a `#[cfg(test)]` unit module. Either shape is fine; this one
//! was chosen because the task specifies `tests/grant_redis_url.rs` as an integration
//! test.

/// The `--grant-redis` URL parser.
///
/// See [`crate::redis_cap::url::parse`] for arguments, return value, errors, and
/// examples. Re-exported under the `main_caps` name so CLI wiring reads consistently
/// with `main_caps::db::parse_url`.
pub(super) use crate::redis_cap::url::parse as parse_url;
