//! Argument coercion and the `304 Not Modified` response map.
//!
//! The response shape matches what `http_serve` already consumes and what
//! `examples/the reference application/server/response.tether` builds by hand: a map with
//! `status`, `headers`, and `body`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{etag_cache, etag_tag};
use crate::value::Value;

/// Coerce one argument to a str, naming the builtin and parameter on failure.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok(text.to_string()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// `etag_of(body)` — strong validator.
pub(super) fn of(args: &[Value]) -> Result<Value, String> {
    let body = str_arg(&args[0], "etag_of: body")?;
    Ok(Value::Str(Rc::new(etag_tag::strong(body.as_bytes()))))
}

/// `etag_weak(body)` — weak validator.
pub(super) fn weak(args: &[Value]) -> Result<Value, String> {
    let body = str_arg(&args[0], "etag_weak: body")?;
    Ok(Value::Str(Rc::new(etag_tag::weak(body.as_bytes()))))
}

/// `etag_matches(if_none_match, etag)` — conditional-request test.
pub(super) fn matches(args: &[Value]) -> Result<Value, String> {
    let header = str_arg(&args[0], "etag_matches: if_none_match")?;
    let etag = str_arg(&args[1], "etag_matches: etag")?;
    Ok(Value::Bool(etag_tag::matches(&header, &etag)))
}

/// `cache_control(options)` — header value from an options map.
pub(super) fn cache_control(args: &[Value]) -> Result<Value, String> {
    let Value::Map(opts) = &args[0] else {
        return Err(format!(
            "cache_control: options must be a map, got {}",
            args[0].type_name()
        ));
    };
    let header = etag_cache::build(&opts.borrow())?;
    Ok(Value::Str(Rc::new(header)))
}

/// `not_modified_response()` — a `304` with an empty body.
///
/// RFC 9110 forbids a body on `304`, so `body` is the empty string rather than
/// omitted: the server writes it verbatim, and omitting the key would make the
/// map shape differ from every other response.
pub(super) fn not_modified() -> Value {
    let mut response = HashMap::new();
    response.insert("status".into(), Value::Int(304));
    response.insert(
        "headers".into(),
        Value::Map(Rc::new(RefCell::new(HashMap::new()))),
    );
    response.insert("body".into(), Value::Str(Rc::new(String::new())));
    Value::Map(Rc::new(RefCell::new(response)))
}
