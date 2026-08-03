//! Resource bounds and the default clock-skew tolerance for RS256 tokens.
//!
//! One responsibility: hold every number this module refuses to exceed, so a
//! deployment can audit "how much work can an unauthenticated stranger make us
//! do" by reading one short file.
//!
//! # Why bounds at all
//!
//! A JWT arrives in an `Authorization` header from an *unauthenticated* client.
//! Every byte of it is attacker-chosen, and all of the shape work — base64url
//! decoding, JSON parsing, role extraction — happens on data nobody has
//! authenticated yet. Without limits, a single request can ask us to decode a
//! 500 MiB segment or materialise a million role strings. Each constant below is
//! a refusal to allocate unbounded memory on a stranger's say-so.
//!
//! # The numbers, and why these numbers
//!
//! | Limit | Value | Rationale |
//! |---|---|---|
//! | [`MAX_TOKEN_BYTES`] | 8192 | A Keycloak access token with ~30 realm roles and three resource clients is ~2.5 KiB; 8 KiB is also the usual HTTP header line limit, so a larger token could not be delivered anyway. |
//! | [`MAX_ROLES`] | 256 | Realms grant roles, they do not enumerate the power set. 256 is far past any real RBAC model and still bounded. |
//! | [`MAX_RESOURCE_CLIENTS`] | 64 | `resource_access` has one entry per OAuth client the token is scoped to; a token scoped to 64 clients is already a misconfiguration. |
//! | [`MAX_AUDIENCES`] | 32 | `aud` as an array is legal (RFC 7519 §4.1.3) and short in practice. |
//! | [`DEFAULT_SKEW_SECS`] | 60 | See below. |
//!
//! # Why 60 seconds of skew, and why symmetric
//!
//! The issuer's clock and ours are different clocks. NTP-synchronised hosts
//! normally agree within milliseconds, but a host that has just booted, a VM
//! resumed from a snapshot, or a container on a busy hypervisor can drift by
//! tens of seconds. Zero tolerance turns that ordinary drift into intermittent
//! `401`s that reproduce for nobody.
//!
//! 60 seconds is chosen because it is larger than realistic NTP drift and far
//! smaller than a token lifetime (Keycloak defaults to 300 seconds), so the
//! tolerance can never come close to doubling a token's usable life. Applying it
//! **symmetrically** — added to `exp`, subtracted from `nbf` — means the same
//! single number describes the whole uncertainty about clock offset in either
//! direction. An asymmetric skew would silently encode a belief about *which
//! way* the clocks are wrong, which nobody actually knows.

/// Largest compact serialization this module will look at, in bytes.
pub const MAX_TOKEN_BYTES: usize = 8192;

/// Largest number of role strings parsed from any single role array.
pub const MAX_ROLES: usize = 256;

/// Largest number of `resource_access` clients parsed.
pub const MAX_RESOURCE_CLIENTS: usize = 64;

/// Largest number of `aud` entries parsed.
pub const MAX_AUDIENCES: usize = 32;

/// Default clock-skew tolerance in seconds, applied symmetrically.
pub const DEFAULT_SKEW_SECS: i64 = 60;
