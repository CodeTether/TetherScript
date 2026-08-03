//! The request-context map, and the standalone request-id extraction.
//!
//! One extraction, one shape, computed once per request. The point is that a
//! handler never re-derives any of these fields: the reference application's
//! `IdentityMiddleware` produces a context object before the route runs, and a
//! handler that reads `ctx.client_ip` cannot disagree with a sibling handler about
//! what the client's address is.
//!
//! Optional headers are represented as `nil`, never as `""`. An empty string is
//! indistinguishable from a header that arrived empty, and a log line reading
//! `user_agent=` is ambiguous in a way `user_agent=nil` is not.
//!
//! Field derivation lives in [`super::identity_context_fields`]; this module owns
//! only the assembly.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::identity_context_fields as fields;
use super::identity_headers::{as_map, str_field};
use super::identity_secure;
use crate::value::Value;

/// Build the context map for a request.
///
/// # Arguments
///
/// * `request` — The request map `http_serve` hands a handler: `method`, `path`,
///   `query`, `headers`, `body`.
///
/// # Returns
///
/// A map with `method`, `path`, `query`, `client_ip`, `user_agent`, `referer`,
/// `request_id`, and `is_secure`. `user_agent` and `referer` are `nil` when the
/// header is absent, so a request carrying no optional headers still yields a
/// complete context rather than an error.
///
/// # Errors
///
/// Returns an error naming the missing or mistyped field when `request` is not a
/// map, when `headers` is present but not a map, or when `method`, `path`, or
/// `query` is absent or not a str. Those three are always present on a real request
/// map, so their absence means the caller passed something that is not a request,
/// and guessing a default would hide that.
pub(super) fn extract(request: &Value) -> Result<Value, String> {
    let req = as_map(request, "request_context: request")?;
    let headers = headers_of(&req, "request_context")?;

    let mut ctx = HashMap::new();
    for field in ["method", "path", "query"] {
        let text = str_field(&req, field, "request_context")?;
        ctx.insert(field.to_string(), Value::Str(Rc::new(text)));
    }
    let address = fields::client_ip(&req, &headers);
    ctx.insert("client_ip".into(), Value::Str(Rc::new(address)));
    ctx.insert(
        "user_agent".into(),
        fields::optional(&headers, "user-agent"),
    );
    ctx.insert("referer".into(), fields::optional(&headers, "referer"));
    let id = fields::request_id(&headers);
    ctx.insert("request_id".into(), Value::Str(Rc::new(id)));
    let secure = identity_secure::is_secure(&headers);
    ctx.insert("is_secure".into(), Value::Bool(secure));
    Ok(Value::Map(Rc::new(RefCell::new(ctx))))
}

/// The safe request id for a request, without building a whole context.
///
/// # Arguments
///
/// * `request` — The request map.
///
/// # Returns
///
/// The incoming `X-Request-ID` when it passes validation, otherwise a fresh one.
///
/// # Errors
///
/// Returns an error naming the actual type when `request` is not a map, or when
/// `headers` is present but is not a map.
pub(super) fn id_of(request: &Value) -> Result<Value, String> {
    let req = as_map(request, "request_id: request")?;
    let headers = headers_of(&req, "request_id")?;
    Ok(Value::Str(Rc::new(fields::request_id(&headers))))
}

/// The request's header map, empty when the field is absent.
///
/// Absent `headers` is tolerated so a hand-built request fixture still yields a
/// context; a *mistyped* `headers` is not, because that is a real shape error.
fn headers_of(req: &HashMap<String, Value>, label: &str) -> Result<HashMap<String, Value>, String> {
    match req.get("headers") {
        Some(value) => as_map(value, &format!("{label}: headers")),
        None => Ok(HashMap::new()),
    }
}
