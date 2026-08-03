//! Environment registration for the request-building and callback built-ins.
//!
//! Split from [`super::install_pkce`] purely to respect the 50-line file limit;
//! [`super::install`] calls both.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::args::{map_arg, str_arg};
use super::callback::parse::params;
use super::request::{body, url};
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Wrap a `Result<String, String>` as a tetherscript `Result` of a str.
fn text(result: Result<String, String>) -> Value {
    wrap(result.map(|rendered| Value::Str(Rc::new(rendered))))
}

/// Define `oauth_authorize_url`, `oauth_token_request_body`, and
/// `oauth_callback_params` in `env`.
///
/// # Arguments
///
/// * `env` — Global environment the interpreter is populating.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "oauth_authorize_url",
        pure_native("oauth_authorize_url", Some(1), |args| {
            let config = map_arg(&args[0], "oauth_authorize_url: config")?;
            Ok(text(url::build(&config)))
        }),
        false,
    );
    bindings.define(
        "oauth_token_request_body",
        pure_native("oauth_token_request_body", Some(3), |args| {
            let config = map_arg(&args[0], "oauth_token_request_body: config")?;
            let code = str_arg(&args[1], "oauth_token_request_body: code")?;
            let verifier = str_arg(&args[2], "oauth_token_request_body: code_verifier")?;
            Ok(text(body::build(&config, &code, &verifier)))
        }),
        false,
    );
    bindings.define(
        "oauth_callback_params",
        pure_native("oauth_callback_params", Some(1), |args| {
            let query = str_arg(&args[0], "oauth_callback_params: query_string")?;
            Ok(wrap(params(&query)))
        }),
        false,
    );
}
