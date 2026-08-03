//! Redis-backed session store and durable rate limiter — the pure logic half.
//!
//! # What gap this closes
//!
//! Two existing groups each stop short of what a real deployment needs:
//!
//! 1. [`super::session`] signs a cookie. Signing proves the payload was not
//!    *edited*; it does not hide it. Anyone holding the cookie — the user, an
//!    extension, a proxy log, anyone who copies it — can decode the payload, and the
//!    server cannot revoke it before its own `exp`. Keeping session state on the
//!    server and handing the client nothing but an opaque id fixes both at once.
//! 2. [`super::ratelimit`] is a token bucket whose state the caller owns, so it lives
//!    in one process's memory: it resets on restart and each worker behind a load
//!    balancer grants a separate full allowance. A counter keyed by subject and
//!    window, held in Redis, is shared by every process and survives a restart.
//!
//! # This group does not talk to Redis
//!
//! Every built-in here takes and returns plain values: key derivation,
//! serialization, id generation, and fixed-window arithmetic. The transport —
//! connection, `GET`/`SETEX`/`INCR`/`EXPIRE`, and failure policy — belongs to the
//! Redis capability. Keeping the two apart means this logic is unit-testable with no
//! server running, and the same functions serve any backend.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `session_store_key(prefix, session_id)` | `Result` of the namespaced key str |
//! | `session_store_encode(payload_map)` | `Result` of the serialized str |
//! | `session_store_decode(text)` | `Result` of the payload map |
//! | `session_store_new_id()` | a fresh 64-char hex id str |
//! | `session_rotate_id(old_id)` | `Result` of a different id str |
//! | `ratelimit_window_key(prefix, subject, window_secs, now_secs)` | `Result` of the bucket key str |
//! | `ratelimit_window_verdict(count, limit, window_secs, now_secs)` | `Result` of the verdict map |
//!
//! # Examples
//!
//! ```tether
//! // Session: the client gets the id, Redis gets the payload.
//! let id = session_store_new_id()
//! let key = session_store_key("sess", id)?
//! let data = map()
//! data.uid = 42
//! data.name = "Ada;Lovelace"          // separators survive the round-trip
//! redis.setex(key, 1800, session_store_encode(data)?)?
//!
//! // On login, rotate: the pre-login id must stop naming a session.
//! let next = session_rotate_id(id)?
//! redis.rename(key, session_store_key("sess", next)?)?
//!
//! // Rate limit: one INCR against a key every process agrees on.
//! let now = time_now_ms() / 1000
//! let bucket = ratelimit_window_key("rl", client_ip(req.headers, req.remote_addr), 60, now)?
//! let count = redis.incr(bucket)?
//! let verdict = ratelimit_window_verdict(count, 100, 60, now)?
//! if !verdict.allowed { return too_many_requests_response(verdict.retry_after_secs * 1000) }
//! ```
//!
//! # Security summary
//!
//! * **Guessing.** An id is 256 bits of OS entropy — see [`sessionstore_id`].
//! * **Key injection.** A cookie-supplied id containing `:` could address another
//!   namespace's key, so it is rejected — see [`sessionstore_validate`].
//! * **Session fixation.** Rotation on privilege change is the fix — see
//!   [`sessionstore_rotate`].
//! * **Burst.** A fixed window admits up to `2 x limit` across a boundary, stated
//!   honestly in [`sessionstore_window`].
//! * **Lossless encoding.** Separators and newlines inside values are escaped, so a
//!   naive split cannot corrupt data — see [`sessionstore_escape`].
//!
//! # Layout
//!
//! * `sessionstore_validate` — untrusted key components, and key injection
//! * `sessionstore_key` / `sessionstore_windowkey` — the two key derivations
//! * `sessionstore_entropy` / `sessionstore_id` / `sessionstore_rotate` — ids
//! * `sessionstore_escape` / `sessionstore_unescape` — reversible escaping
//! * `sessionstore_tag` / `sessionstore_untag` — value type tags
//! * `sessionstore_encode` / `sessionstore_decode` — the wire format
//! * `sessionstore_window` / `sessionstore_verdict` — fixed-window arithmetic
//! * `sessionstore_args` — argument coercion with named errors
//! * `sessionstore_ops_*` — one built-in body per family
//! * `sessionstore_install` — registration

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "sessionstore_args.rs"]
mod sessionstore_args;
#[path = "sessionstore_decode.rs"]
mod sessionstore_decode;
#[path = "sessionstore_encode.rs"]
mod sessionstore_encode;
// A local CSPRNG read rather than an import: the `random` group's equivalent is
// `mod`-private and this group may not edit it. See the file for the one-line
// consolidation the integrator can take.
#[path = "sessionstore_entropy.rs"]
mod sessionstore_entropy;
#[path = "sessionstore_escape.rs"]
mod sessionstore_escape;
#[path = "sessionstore_id.rs"]
mod sessionstore_id;
#[path = "sessionstore_install.rs"]
mod sessionstore_install;
#[path = "sessionstore_key.rs"]
mod sessionstore_key;
#[path = "sessionstore_ops_limit.rs"]
mod sessionstore_ops_limit;
#[path = "sessionstore_ops_session.rs"]
mod sessionstore_ops_session;
#[path = "sessionstore_rotate.rs"]
mod sessionstore_rotate;
#[path = "sessionstore_tag.rs"]
mod sessionstore_tag;
#[path = "sessionstore_unescape.rs"]
mod sessionstore_unescape;
#[path = "sessionstore_untag.rs"]
mod sessionstore_untag;
#[path = "sessionstore_validate.rs"]
mod sessionstore_validate;
#[path = "sessionstore_verdict.rs"]
mod sessionstore_verdict;
#[path = "sessionstore_window.rs"]
mod sessionstore_window;
#[path = "sessionstore_windowkey.rs"]
mod sessionstore_windowkey;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global interpreter environment being populated.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    sessionstore_install::install(env);
}
