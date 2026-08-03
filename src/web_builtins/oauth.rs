//! OAuth 2.0 authorization-code flow with PKCE, as tetherscript built-ins.
//!
//! Owner: sub-agent `oauth_pkce`. The reference application authenticates against
//! Keycloak with the authorization-code flow. This group owns the two halves a script
//! cannot safely hand-roll: **building the authorization request** and **validating the
//! callback**. It deliberately does *not* verify token signatures — JWKS parsing and RSA
//! verification are separate groups — so nothing here should be read as authenticating
//! an `id_token`.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `oauth_pkce_pair()` | `Result` map: `code_verifier`, `code_challenge`, `code_challenge_method` |
//! | `oauth_pkce_challenge(verifier)` | `Result` str: the S256 challenge for an existing verifier |
//! | `oauth_authorize_url(config)` | `Result` str: the full authorization URL |
//! | `oauth_state_token(secret, ttl_secs, return_to)` | `Result` str: signed, expiring state |
//! | `oauth_state_verify(secret, token)` | `Result` str: the validated return path |
//! | `oauth_token_request_body(config, code, verifier)` | `Result` str: form-encoded body |
//! | `oauth_callback_params(query)` | `Result` map: `code`, `state`, `error`, `error_description` |
//!
//! # Security rationale
//!
//! Every rule enforced here exists because omitting it is a known attack. The reasoning
//! is documented beside the code that enforces it:
//!
//! * **S256, never `plain`** — [`pkce`]. The `plain` method sends the verifier itself, so
//!   an intercepted authorization code stays redeemable.
//! * **Signed, expiring state** — [`state`]. Unsigned state is a CSRF hole in the
//!   callback; omitting state altogether permits authorization-code injection.
//! * **Relative `return_to` only** — [`return_to`]. Absolute URLs, scheme-relative
//!   `//host`, and backslash forms are all open redirects.
//! * **Exact `redirect_uri`, never a prefix** — [`request`]. Prefix matching lets an
//!   attacker-controlled suffix receive the authorization code.
//! * **An error callback is never a success** — [`callback`]. Reading only `code` and
//!   ignoring `error` defers a clear failure into a confusing one.
//! * **No client secret in the authorization URL** — [`request`]. It lands in browser
//!   history and access logs; it belongs only in the token-request body.
//!
//! # Examples
//!
//! Maps are built with `map()` and field assignment; tetherscript has no map literal.
//!
//! ```tether
//! fn login_redirect(secret) {
//!     let pkce = oauth_pkce_pair()?
//!     let config = map()
//!     config.authorize_url = "https://sso.example.com/realms/app/protocol/openid-connect/auth"
//!     config.client_id = "web"
//!     config.redirect_uri = "https://app.example.com/callback"
//!     config.scope = "openid profile email"
//!     config.state = oauth_state_token(secret, 600, "/dashboard")?
//!     config.code_challenge = pkce.code_challenge
//!     // Store pkce.code_verifier in the user's session; it is needed on callback.
//!     oauth_authorize_url(config)
//! }
//!
//! fn on_callback(secret, query) {
//!     // Err here when the provider reported `error`, never a silent missing code.
//!     let params = oauth_callback_params(query)?
//!     let return_to = oauth_state_verify(secret, params.state)?
//!     return_to
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

// Explicit paths throughout: this file is included with `#[path]` from
// `web_builtins.rs`, so relative submodule resolution would look outside
// `web_builtins/`. Deeper concerns are declared by their own parent module —
// `pkce`, `state`, `request`, `callback`, `codec`, and `percent` each own their
// splits — which is what keeps this list short.
#[path = "oauth_args.rs"]
pub(crate) mod args;
#[path = "oauth_callback.rs"]
pub(crate) mod callback;
#[path = "oauth_clock.rs"]
pub(crate) mod clock;
#[path = "oauth_codec.rs"]
pub(crate) mod codec;
#[path = "oauth_entropy.rs"]
pub(crate) mod entropy;
#[path = "oauth_install_pkce.rs"]
pub(crate) mod install_pkce;
#[path = "oauth_install_request.rs"]
pub(crate) mod install_request;
#[path = "oauth_percent.rs"]
pub(crate) mod percent;
#[path = "oauth_pkce.rs"]
pub(crate) mod pkce;
#[path = "oauth_request.rs"]
pub(crate) mod request;
#[path = "oauth_return_to.rs"]
pub(crate) mod return_to;
#[path = "oauth_state.rs"]
pub(crate) mod state;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
///
/// # Returns
///
/// Nothing; the seven `oauth_*` built-ins are defined in `env` as immutable bindings.
/// The registration is split across two files only because seven definitions do not fit
/// in one 50-line file.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    install_pkce::install(env);
    install_request::install(env);
}
