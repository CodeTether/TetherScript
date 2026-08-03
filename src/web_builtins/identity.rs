//! Identity and request-context extraction for request handlers.
//!
//! The reference application registers an `IdentityMiddleware` plus a session-IP
//! tracker in its Actix `create_app`, so every handler receives an already-derived
//! caller identity. The port had no equivalent: each route re-derived who the caller
//! was from raw headers and claims. That is how an authorisation check gets missed —
//! the twentieth handler forgets the role test, and nothing fails.
//!
//! This group turns that per-handler ritual into a few primitives: extract the
//! request context once, turn verified claims into an identity once, ask for a role,
//! and demand a role.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `request_context(request)` | `Result` of a context map |
//! | `request_id(request)` | `Result` of a safe request id str |
//! | `identity_from_claims(claims)` | `Result` of an identity map |
//! | `anonymous()` | the identity map of an unauthenticated caller |
//! | `has_role(identity, role)` | bool, exact match |
//! | `require_role(identity, role)` | `Result` of `nil`, or a 403 response map |
//! | `ip_changed(session_context, current_ip)` | bool |
//!
//! The context map carries `method`, `path`, `query`, `client_ip`, `user_agent`,
//! `referer`, `request_id`, and `is_secure`. The identity map carries `subject`,
//! `roles`, and `authenticated`.
//!
//! # Examples
//!
//! ```tether
//! fn handle(req) {
//!     let ctx = request_context(req)?
//!     log_line("[" + ctx.request_id + "] " + ctx.method + " " + ctx.path)
//!
//!     let token = bearer_token(req.headers)
//!     let who = if token.is_ok() {
//!         identity_from_claims(jwt_verify(token.unwrap(), secret)?)?
//!     } else {
//!         anonymous()
//!     }
//!
//!     if !who.authenticated { return unauthorized() }
//!     let denied = require_role(who, "admin")?
//!     if denied != nil { return denied }
//!
//!     if ip_changed(session, ctx.client_ip) {
//!         log_line("session address changed; re-authenticating")
//!     }
//! }
//! ```
//!
//! # Relationship to the `header` group
//!
//! `header_get`, `bearer_token`, and `client_ip` already exist and are **not**
//! reimplemented here. Credential *extraction* stays in `bearer_token`, and this
//! group starts from claims some other layer already verified, because deciding
//! whether a token is authentic is a signature concern, not an identity one.
//!
//! The proxy-address precedence this group applies for `client_ip` is the same rule
//! `header::header_client_ip::resolve` applies — leftmost `X-Forwarded-For`, then
//! `X-Real-IP`, then the peer address. It is restated by a local helper only because
//! that function and `header_lookup::find` are `pub(super)` *within the `header`
//! module*, so they are unreachable from a sibling group, and `header.rs` may not be
//! edited to widen them. See [`identity_headers`] for that note in place.
//!
//! # Security
//!
//! Each of the following is the mechanism of a real privilege-escalation class:
//!
//! * A caller-supplied `X-Request-ID` is validated before it is echoed, because it
//!   lands in logs. See [`identity_request_id`].
//! * `identity_from_claims` is anonymous-by-default *structurally*. See
//!   [`identity_shape`].
//! * `has_role` compares whole strings, never prefixes. See [`identity_roles`].
//! * A `roles` claim must be a list of strings. See [`identity_roles_claim`].
//! * `is_secure` reads a client-controlled header. See [`identity_secure`].
//! * `ip_changed` is a signal, not a verdict. See [`identity_session_ip`].
//! * `require_role` answers 403, never 401. See [`identity_response`].
//!
//! # Layout
//!
//! * `identity_headers` — argument coercion and case-insensitive lookup
//! * `identity_request_id` — charset validation of an incoming id
//! * `identity_request_id_gen` — fresh id generation
//! * `identity_secure` — `X-Forwarded-Proto` interpretation
//! * `identity_context_fields` — per-field derivation from the request map
//! * `identity_context` — the context map and standalone id extraction
//! * `identity_shape` — the identity map, and the anonymous-by-default invariant
//! * `identity_claims` — claims to identity
//! * `identity_roles_claim` — the strict `roles` list shape
//! * `identity_roles` — exact role membership
//! * `identity_gate` — the `require_role` decision
//! * `identity_response` — the 403 response map
//! * `identity_session_ip` — the session address-change signal
//! * `identity_install` — context built-in registration
//! * `identity_install_auth` — identity/role built-in registration

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "identity_claims.rs"]
mod identity_claims;
#[path = "identity_context.rs"]
mod identity_context;
#[path = "identity_context_fields.rs"]
mod identity_context_fields;
#[path = "identity_gate.rs"]
mod identity_gate;
#[path = "identity_headers.rs"]
mod identity_headers;
#[path = "identity_install.rs"]
mod identity_install;
#[path = "identity_install_auth.rs"]
mod identity_install_auth;
#[path = "identity_request_id.rs"]
mod identity_request_id;
#[path = "identity_request_id_gen.rs"]
mod identity_request_id_gen;
#[path = "identity_response.rs"]
mod identity_response;
#[path = "identity_roles.rs"]
mod identity_roles;
#[path = "identity_roles_claim.rs"]
mod identity_roles_claim;
#[path = "identity_secure.rs"]
mod identity_secure;
#[path = "identity_session_ip.rs"]
mod identity_session_ip;
#[path = "identity_shape.rs"]
mod identity_shape;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — The global environment the interpreter is building.
///
/// # Returns
///
/// Nothing. Defines `request_context`, `request_id`, `identity_from_claims`,
/// `anonymous`, `has_role`, `require_role`, and `ip_changed` in `env`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    identity_install::install(env);
}
