//! Environment registration for the PKCE and state built-ins.
//!
//! Split from [`super::install_request`] purely to respect the 50-line file limit;
//! [`super::install`] calls both so the group still has one obvious entry point.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::args::{int_arg, str_arg};
use super::pkce::gen::challenge;
use super::pkce::pair::build as pkce_pair;
use super::state::{mint, verify};
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `oauth_pkce_pair`, `oauth_pkce_challenge`, `oauth_state_token`, and
/// `oauth_state_verify` in `env`.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "oauth_pkce_pair",
        pure_native("oauth_pkce_pair", Some(0), |_args| Ok(wrap(pkce_pair()))),
        false,
    );
    bindings.define(
        "oauth_pkce_challenge",
        pure_native("oauth_pkce_challenge", Some(1), |args| {
            let verifier = str_arg(&args[0], "oauth_pkce_challenge: code_verifier")?;
            let derived = challenge(&verifier).map(|text| Value::Str(Rc::new(text)));
            Ok(wrap(derived))
        }),
        false,
    );
    bindings.define(
        "oauth_state_token",
        pure_native("oauth_state_token", Some(3), |args| {
            let secret = str_arg(&args[0], "oauth_state_token: secret")?;
            let ttl = int_arg(&args[1], "oauth_state_token: ttl_secs")?;
            let return_to = str_arg(&args[2], "oauth_state_token: return_to")?;
            Ok(wrap(mint::token(&secret, ttl, &return_to)))
        }),
        false,
    );
    bindings.define(
        "oauth_state_verify",
        pure_native("oauth_state_verify", Some(2), |args| {
            let secret = str_arg(&args[0], "oauth_state_verify: secret")?;
            let token = str_arg(&args[1], "oauth_state_verify: token")?;
            Ok(wrap(verify::verify(&secret, &token)))
        }),
        false,
    );
}
