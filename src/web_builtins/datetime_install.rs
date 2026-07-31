//! Environment registration for the date/time built-ins.

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::datetime_args::{int_arg, now_secs, str_arg};
use super::datetime_format::{http_date, rfc3339};
use super::datetime_parse::{http_date_parse, rfc3339_parse};
use crate::system::result_value;
use crate::value::{Env, Value};

/// Define every date/time built-in in `env`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define("http_date", format_builtin("http_date", http_date), false);
    bindings.define("rfc3339", format_builtin("rfc3339", rfc3339), false);
    bindings.define(
        "http_date_parse",
        parse_builtin("http_date_parse", http_date_parse),
        false,
    );
    bindings.define(
        "rfc3339_parse",
        parse_builtin("rfc3339_parse", rfc3339_parse),
        false,
    );
    bindings.define(
        "time_now_secs",
        pure_native("time_now_secs", Some(0), |_args| Ok(Value::Int(now_secs()))),
        false,
    );
}

/// Build a seconds-to-string built-in.
fn format_builtin(name: &'static str, render: fn(i64) -> String) -> Value {
    pure_native(name, Some(1), move |args| {
        let label = format!("{name}: seconds");
        Ok(result_value(
            int_arg(&args[0], &label).map(|seconds| Value::Str(Rc::new(render(seconds)))),
        ))
    })
}

/// Build a string-to-seconds built-in.
fn parse_builtin(name: &'static str, parse: fn(&str) -> Result<i64, String>) -> Value {
    pure_native(name, Some(1), move |args| {
        let label = format!("{name}: text");
        Ok(result_value(
            str_arg(&args[0], &label)
                .and_then(|text| parse(&text))
                .map(Value::Int),
        ))
    })
}
