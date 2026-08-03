//! Server-side session store with a pluggable backend.
//!
//! This is the *other* half of [`super::session`]. That group signs a payload the
//! client carries; this group keeps the payload on the server and hands the client
//! nothing but an opaque id. Three gaps make the difference load-bearing, and they
//! are the reason this group exists:
//!
//! 1. **Confidentiality.** `session_sign` produces a signed but **not encrypted**
//!    value — `docs/web-builtins.md` states this explicitly — so anything in a
//!    signed cookie is readable by the client and by anyone who copies it. A stored
//!    session can hold an internal user id, an entitlement set, or an upstream
//!    token, none of which may be disclosed.
//! 2. **Revocation.** A signed cookie is valid until its own `exp`; the server has
//!    no say. Deleting a stored record makes a still-unexpired cookie useless at
//!    once, which is what "log out everywhere" and post-password-change
//!    invalidation require. See [`store_destroy`].
//! 3. **Size.** A cookie is capped near 4 KB in practice. A stored session is
//!    bounded only by the backend.
//!
//! # The cookie carries the id and nothing else
//!
//! The id is a 64-character hex string; the payload never leaves the process. Put
//! the id in a cookie with `cookie_serialize` — hex passes its injection guard
//! unchanged — and, if you want tamper-evidence on the pointer itself, sign that
//! *id* with `session_sign`. Never put session data in the cookie: it would be
//! readable, unrevocable, and size-bounded again, which is all three gaps back.
//!
//! # Two clocks
//!
//! Idle timeout bounds an *abandoned* session and resets on activity; absolute
//! lifetime bounds a *stolen* one and never resets. Either alone is insufficient —
//! see [`store_record`] for the argument. `store_touch` extends the first and
//! deliberately not the second.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `store_create(subject, data)` | `Result` of the session map |
//! | `store_load(id)` | `Result` of the session map |
//! | `store_save(id, data)` | `Result` of the saved session map |
//! | `store_touch(id)` | `Result` of the session map, idle clock reset |
//! | `store_rotate(id)` | `Result` of the session under a **new** id |
//! | `store_destroy(id)` | `Result` of bool |
//! | `store_destroy_subject(subject)` | `Result` of the count removed |
//! | `store_configure(idle_ttl_ms, absolute_ttl_ms)` | `Result` of nil |
//! | `store_sweep()` | `Result` of the count of expired records dropped |
//! | `store_count()` | `Result` of the record count |
//!
//! # Examples
//!
//! ```tether
//! store_configure(1800000, 604800000).unwrap()      // 30 min idle, 7 day ceiling
//!
//! let anon = store_create("anonymous", map()).unwrap()
//! // ... user logs in: rotate to defeat session fixation, data survives
//! let session = store_rotate(anon.id).unwrap()
//!
//! let opts = map()
//! opts.http_only = true
//! opts.same_site = "Lax"
//! opts.path = "/"
//! println(cookie_serialize("sid", session.id, opts).unwrap())
//!
//! store_destroy(session.id).unwrap()                // the cookie is now useless
//! ```
//!
//! # Layout
//!
//! * `store_backend` — the trait a Redis, SQL, or in-memory backend implements
//! * `store_memory` — the in-memory backend, with its limitations documented
//! * `store_record` — the stored record and why there are two clocks
//! * `store_expiry` — expiry evaluation and its error wording
//! * `store_id` — id generation and the entropy argument, in bits
//! * `store_entropy` — the CSPRNG byte source behind `store_id`
//! * `store_compare` — constant-time comparison, and where it is not needed
//! * `store_clock` — the millisecond clock
//! * `store_lookup` — the one read path that enforces expiry
//! * `store_create` — creation and id rotation (session fixation)
//! * `store_write` — touch and save
//! * `store_destroy` — revocation
//! * `store_state` — the process-wide instance and the TTL policy
//! * `store_fields` / `store_args` — the script-visible shape and coercion
//! * `store_ops_*` — one built-in body per operation family
//! * `store_install` — registration

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "store_args.rs"]
mod store_args;
#[path = "store_backend.rs"]
mod store_backend;
#[path = "store_clock.rs"]
mod store_clock;
#[path = "store_compare.rs"]
mod store_compare;
#[path = "store_create.rs"]
mod store_create;
#[path = "store_destroy.rs"]
mod store_destroy;
// A local CSPRNG read rather than an import: the `random` group's equivalent is
// `mod`-private and this group may not edit it. See the file for the one-line
// consolidation the integrator can take.
#[path = "store_entropy.rs"]
mod store_entropy;
#[path = "store_expiry.rs"]
mod store_expiry;
#[path = "store_fields.rs"]
mod store_fields;
#[path = "store_id.rs"]
mod store_id;
#[path = "store_install.rs"]
mod store_install;
#[path = "store_lookup.rs"]
mod store_lookup;
#[path = "store_memory.rs"]
mod store_memory;
#[path = "store_ops_access.rs"]
mod store_ops_access;
#[path = "store_ops_config.rs"]
mod store_ops_config;
#[path = "store_ops_mint.rs"]
mod store_ops_mint;
#[path = "store_ops_revoke.rs"]
mod store_ops_revoke;
#[path = "store_record.rs"]
mod store_record;
#[path = "store_state.rs"]
mod store_state;
#[path = "store_write.rs"]
mod store_write;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global interpreter environment being populated.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    store_install::install(env);
}
