//! Environment registration for the form built-ins.
//!
//! Separated from `form.rs` so the owning module only declares its submodules,
//! keeping every file inside the 50-line limit.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::{form_codec, form_pairs};
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define `url_encode`, `url_decode`, `form_parse`, and `form_encode`.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("url_encode", url_encode(), false);
    bindings.define("url_decode", url_decode(), false);
    bindings.define("form_parse", form_parse(), false);
    bindings.define("form_encode", form_encode(), false);
}

fn url_encode() -> Value {
    pure_native("url_encode", Some(1), |args| {
        let input = str_arg(&args[0], "url_encode: input")?;
        Ok(Value::Str(Rc::new(form_codec::encode(&input))))
    })
}

fn url_decode() -> Value {
    pure_native("url_decode", Some(1), |args| {
        let input = str_arg(&args[0], "url_decode: input")?;
        Ok(result_value(
            form_codec::decode(&input, "url_decode").map(|text| Value::Str(Rc::new(text))),
        ))
    })
}

fn form_parse() -> Value {
    pure_native("form_parse", Some(1), |args| {
        let input = str_arg(&args[0], "form_parse: input")?;
        Ok(result_value(form_pairs::parse(&input)))
    })
}

fn form_encode() -> Value {
    pure_native("form_encode", Some(1), |args| match &args[0] {
        Value::Map(map) => Ok(result_value(
            form_pairs::encode_map(&map.borrow()).map(|text| Value::Str(Rc::new(text))),
        )),
        other => Err(format!(
            "form_encode: input must be map, got {}",
            other.type_name()
        )),
    })
}

/// Require a str argument, naming the built-in and parameter on mismatch.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
