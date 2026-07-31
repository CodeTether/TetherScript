//! Cookie built-ins.
//!
//! Provides `cookie_parse` for request `Cookie` headers and `cookie_serialize`
//! for `Set-Cookie` response headers, covering the attributes the the reference application
//! session middleware relies on: `Path`, `Domain`, `Max-Age`, `Expires`,
//! `HttpOnly`, `Secure`, and `SameSite`.
//!
//! # Security
//!
//! `cookie_serialize` refuses to emit a header containing a control character,
//! `;`, or `,` in any position. Those bytes are what turn a cookie value into a
//! second cookie or an entirely new response header, so they are rejected with a
//! named error instead of being stripped. See `cookie_guard`.
//!
//! # Examples
//!
//! ```tether
//! let jar = cookie_parse("id=a1; theme=dark")
//! println(jar["theme"])
//!
//! let opts = map()
//! opts.path = "/"
//! opts.http_only = true
//! opts.same_site = "Lax"
//! opts.max_age = 604800
//! println(cookie_serialize("id", "a1", opts)?)
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use super::super::pure_native;
use crate::system::result_value;
use crate::value::{Env, Value};

#[path = "cookie_alias.rs"]
pub(crate) mod cookie_alias;
#[path = "cookie_guard.rs"]
pub(crate) mod cookie_guard;
#[path = "cookie_options.rs"]
pub(crate) mod cookie_options;
#[path = "cookie_parse.rs"]
pub(crate) mod cookie_parse;
#[path = "cookie_serialize.rs"]
pub(crate) mod cookie_serialize;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "cookie_parse",
        pure_native("cookie_parse", Some(1), |args| {
            Ok(cookie_parse::parse(&cookie_guard::string_arg(
                &args[0],
                "cookie_parse: header",
            )?))
        }),
        false,
    );
    bindings.define(
        "cookie_serialize",
        pure_native("cookie_serialize", Some(3), |args| {
            Ok(result_value(serialize_args(args)))
        }),
        false,
    );
}

/// Coerce script arguments, then delegate to the serializer.
fn serialize_args(args: &[Value]) -> Result<Value, String> {
    let name = cookie_guard::string_arg(&args[0], "cookie_serialize: name")?;
    let value = cookie_guard::string_arg(&args[1], "cookie_serialize: value")?;
    cookie_guard::check_name(&name)?;
    cookie_guard::reject_injection("value", &value)?;
    let Value::Map(opts) = &args[2] else {
        return Err(format!(
            "cookie_serialize: options must be map, got {}",
            args[2].type_name()
        ));
    };
    let header = cookie_serialize::serialize(&name, &value, &opts.borrow())?;
    Ok(Value::Str(Rc::new(header)))
}
