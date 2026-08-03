//! # Epoch shifts between PostgreSQL and Unix time
//!
//! ## The constant that matters
//!
//! PostgreSQL's binary `timestamp`/`timestamptz` counts from **2000-01-01
//! T00:00:00 UTC**, not from the Unix epoch of 1970-01-01, and it counts in
//! **microseconds**, not seconds. Binary `date` counts days from the same 2000
//! epoch.
//!
//! Getting either wrong is silent. Reading the microsecond counter as Unix seconds
//! lands somewhere in the year 24-million; reading microseconds but skipping the
//! 30-year shift lands in 1970 instead of 2000 — a plausible-looking date that
//! passes review. Hence named constants and a pinned test rather than inline
//! literals.
//!
//! The shift is exactly **946 684 800 seconds**: 30 years of 365 days plus the
//! 7 leap days of 1972, 1976, 1980, 1984, 1988, 1992, and 1996, giving 10 957
//! days, and `10_957 * 86_400 == 946_684_800`. `tests/postgres_binary.rs` asserts
//! that identity *and* checks a known instant in both directions.
//!
//! `timestamptz` carries no zone on the wire: the server normalises to UTC and the
//! session `TimeZone` affects only *text* output. Binary decoding is therefore
//! zone-independent, which is a second reason to prefer it.

#[path = "binary_time_civil.rs"]
pub mod civil;

/// Seconds from the Unix epoch to the PostgreSQL epoch, 2000-01-01T00:00:00Z.
///
/// Equals `PG_EPOCH_UNIX_DAYS * 86_400`.
pub const PG_EPOCH_UNIX_SECONDS: i64 = 946_684_800;

/// Microseconds from the Unix epoch to the PostgreSQL epoch.
///
/// Equals `PG_EPOCH_UNIX_SECONDS * 1_000_000`. An `i64` holds roughly ±292 000
/// years of microseconds, so this cannot overflow for any real timestamp.
pub const PG_EPOCH_UNIX_MICROS: i64 = PG_EPOCH_UNIX_SECONDS * 1_000_000;

/// Days from the Unix epoch to the PostgreSQL epoch: 30 years plus 7 leap days.
pub const PG_EPOCH_UNIX_DAYS: i32 = 10_957;

/// Convert a PostgreSQL `timestamp`/`timestamptz` counter to Unix microseconds.
///
/// # Arguments
///
/// * `pg_micros` — microseconds since 2000-01-01, read big-endian off the wire.
///
/// # Returns
///
/// Microseconds since 1970-01-01, saturating rather than wrapping at the `i64`
/// extremes so the `infinity`/`-infinity` sentinels cannot wrap into a
/// plausible-looking date.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::timestamp_unix_micros;
///
/// // 2000-01-01T00:00:00Z is the PostgreSQL zero point.
/// assert_eq!(timestamp_unix_micros(0), 946_684_800_000_000);
/// // 2024-01-15T10:30:00Z
/// assert_eq!(timestamp_unix_micros(758_629_800_000_000), 1_705_314_600_000_000);
/// ```
pub fn timestamp_unix_micros(pg_micros: i64) -> i64 {
    pg_micros.saturating_add(PG_EPOCH_UNIX_MICROS)
}

/// Convert a PostgreSQL `date` counter to days since the Unix epoch.
///
/// # Arguments
///
/// * `pg_days` — days since 2000-01-01, read big-endian off the wire.
///
/// # Returns
///
/// Days since 1970-01-01, saturating at the `i32` extremes.
///
/// # Examples
///
/// ```rust
/// use tetherscript::postgres::binary::date_unix_days;
///
/// assert_eq!(date_unix_days(0), 10_957); // 2000-01-01
/// assert_eq!(date_unix_days(8_780), 19_737); // 2024-01-15
/// ```
pub fn date_unix_days(pg_days: i32) -> i32 {
    pg_days.saturating_add(PG_EPOCH_UNIX_DAYS)
}
