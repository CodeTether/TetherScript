//! Cross-Origin Resource Sharing (CORS) built-ins.
//!
//! The reference application registers a CORS layer once, in Actix's
//! `create_app`, and every handler inherits it. The tetherscript port had no way
//! to answer a preflight or emit the response headers a browser requires, so each
//! handler had to hand-roll the header set — which is exactly how CORS gets got
//! wrong. These built-ins let a script declare policy once and then ask two
//! questions per request: "is this a preflight?" and "what headers do I add?".
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `cors_policy(config)` | `Result` of a validated policy map |
//! | `cors_preflight(policy, request)` | `Result` of `nil`, or a complete 204 response map |
//! | `cors_headers(policy, request)` | `Result` of a map of response headers to merge |
//! | `is_preflight(request)` | bool |
//!
//! `config` accepts `origins` (a list of exact origins, or the string `"*"`),
//! `methods`, `headers`, `expose`, `credentials`, and `max_age`. Any other key is
//! rejected, because a typo such as `origin` would otherwise produce a policy
//! that silently allows nothing.
//!
//! # Examples
//!
//! ```tether
//! fn app_policy() {
//!     let config = map()
//!     config.origins = ["https://app.example.com"]
//!     config.methods = ["GET", "POST"]
//!     config.headers = ["content-type", "authorization"]
//!     config.expose = ["x-request-id"]
//!     config.credentials = true
//!     config.max_age = 600
//!     return cors_policy(config)
//! }
//!
//! fn handle(policy, req) {
//!     let pre = cors_preflight(policy, req)
//!     if pre.is_err() { return text(403, pre.err()) }
//!     let answer = pre.unwrap()
//!     if answer != nil { return answer }
//!
//!     let resp = json(200, "{}")
//!     let extra = cors_headers(policy, req)?
//!     for name in extra.keys() { resp.headers[name] = extra[name] }
//!     return resp
//! }
//! ```
//!
//! # Security
//!
//! CORS is a deliberate same-origin-policy bypass, so a permissive mistake here
//! is a real vulnerability rather than a style problem. The rules enforced are:
//!
//! * **`Access-Control-Allow-Origin: *` with `Access-Control-Allow-Credentials:
//!   true` is rejected at policy-construction time.** The Fetch spec forbids the
//!   pair, and a server that emits it is inviting any site on the internet to
//!   read authenticated responses. Failing in `cors_policy` means the mistake is
//!   caught once at startup rather than being re-decided on every request, where
//!   it could be missed on the path nobody tested. See `cors_config`.
//! * **With credentials the exact requesting origin is echoed, never `*`**, and
//!   `Vary: Origin` is emitted alongside it. Without `Vary`, a shared cache keyed
//!   only on the URL will hand origin A's response — carrying
//!   `Allow-Origin: A` — to origin B, or worse, cache B's rejection and serve it
//!   to A. See `cors_origin`.
//! * **An origin that is not on the allow-list produces no `Allow-Origin` header
//!   at all**, not an empty one and not a wildcard. An empty value is a malformed
//!   header some clients mis-handle; a wildcard fallback would defeat the list.
//! * **Origins compare exactly**: scheme, host, and port, byte for byte. Suffix
//!   matching is never used, because `https://evil-example.com` ends with
//!   `example.com` and `https://example.com.evil.net` contains it, so a suffix or
//!   substring test hands the allow-list to anyone who can register a domain.
//! * **A requested method or header that is not on the allow-list is refused**,
//!   with an error naming it, rather than reflected back. Reflection turns the
//!   allow-list into an echo chamber that permits whatever the caller asks for.
//!
//! # Layout
//!
//! * `cors_install` — built-in registration
//! * `cors_config` plus `cors_config_origins`, `cors_config_lists`,
//!   `cors_config_scalars`, `cors_config_syntax`, `cors_config_build` —
//!   construction-time validation and policy assembly
//! * `cors_policy_read` — reading a validated policy map back into a struct
//! * `cors_request` — request-map access and header lookup
//! * `cors_origin` — the allow-origin decision and the `Vary` rule
//! * `cors_preflight`, `cors_preflight_check`, `cors_preflight_response` — 204s
//! * `cors_response` — headers for an actual response
//! * `cors_args`, `cors_fields`, `cors_shape`, `cors_token` — shared helpers

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "cors_args.rs"]
mod cors_args;
#[path = "cors_config.rs"]
mod cors_config;
#[path = "cors_config_build.rs"]
mod cors_config_build;
#[path = "cors_config_lists.rs"]
mod cors_config_lists;
#[path = "cors_config_origins.rs"]
mod cors_config_origins;
#[path = "cors_config_scalars.rs"]
mod cors_config_scalars;
#[path = "cors_config_syntax.rs"]
mod cors_config_syntax;
#[path = "cors_fields.rs"]
mod cors_fields;
#[path = "cors_install.rs"]
mod cors_install;
#[path = "cors_origin.rs"]
mod cors_origin;
#[path = "cors_policy_read.rs"]
mod cors_policy_read;
#[path = "cors_preflight.rs"]
mod cors_preflight;
#[path = "cors_preflight_check.rs"]
mod cors_preflight_check;
#[path = "cors_preflight_response.rs"]
mod cors_preflight_response;
#[path = "cors_request.rs"]
mod cors_request;
#[path = "cors_response.rs"]
mod cors_response;
#[path = "cors_shape.rs"]
mod cors_shape;
#[path = "cors_token.rs"]
mod cors_token;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — The global environment the interpreter is populating.
///
/// # Returns
///
/// Nothing; `cors_policy`, `cors_preflight`, `cors_headers`, and `is_preflight`
/// are defined in `env` as pure natives.
///
/// # Errors
///
/// Cannot fail: registration only inserts bindings. Every failure mode in this
/// group is reported to the script as an `Err` inside the returned `Result`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    cors_install::install(env);
}
