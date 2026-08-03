//! The stored session record and the two independent expiry rules.
//!
//! # Why two clocks and not one
//!
//! A single timeout cannot express both risks:
//!
//! * **Idle timeout** bounds an *abandoned* session. Someone walks away from a
//!   shared machine with the browser open; the credential must stop working soon
//!   after the human stops using it. This clock therefore resets on activity.
//! * **Absolute lifetime** bounds a *stolen* session. An attacker holding a
//!   copied id keeps using it, so from the idle clock's point of view the session
//!   is perfectly healthy — and stays alive forever. This clock never resets, so
//!   it puts a hard ceiling on how long a leaked id is worth anything.
//!
//! Idle alone leaves a stolen id valid indefinitely. Absolute alone leaves an
//! abandoned session live for the whole ceiling. Both are needed, and they are
//! deliberately separate fields rather than one "expires_at", so the record can
//! say *which* rule ended the session.

use std::collections::HashMap;

use crate::value::Value;

/// One server-side session.
///
/// `data` never travels to the client, so unlike a signed cookie payload it may
/// hold values that must not be readable there.
#[derive(Clone)]
pub(super) struct Record {
    /// CSPRNG session id. The only part a cookie ever carries.
    pub(super) id: String,
    /// Whose session this is; the key `store_destroy_subject` sweeps by.
    pub(super) subject: String,
    /// Application state. Cloned in and out, so the caller cannot alias it.
    pub(super) data: HashMap<String, Value>,
    /// When the session was first created. Never moved forward.
    pub(super) created_ms: i64,
    /// Last activity. Moved forward by `store_touch` and `store_save`.
    pub(super) seen_ms: i64,
    /// Idle window in milliseconds, copied from the store at creation.
    pub(super) idle_ttl_ms: i64,
    /// Absolute ceiling in milliseconds, copied from the store at creation.
    pub(super) absolute_ttl_ms: i64,
}

/// Which rule ended a session.
pub(super) enum Expiry {
    /// No activity within `idle_ttl_ms`.
    Idle,
    /// Alive longer than `absolute_ttl_ms`, however active.
    Absolute,
}
