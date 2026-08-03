//! The 403 response map returned by a failed role gate.
//!
//! # Security: 403, never 401
//!
//! The two statuses answer different questions, and conflating them is both an
//! information leak and a client-side infinite loop:
//!
//! * **401 Unauthorized** means *"I do not know who you are"*. It is a request for
//!   authentication, and RFC 9110 §15.5.2 requires it to carry a
//!   `WWW-Authenticate` header telling the client how to authenticate. A client
//!   receiving 401 is expected to obtain a credential and retry.
//! * **403 Forbidden** means *"I know who you are, and you may not do this"*. No
//!   credential the caller can obtain by retrying will change the answer, so
//!   retrying is pointless and the response carries no challenge.
//!
//! `require_role` is reached only after an identity exists, so its refusal is
//! always the second case. Answering 401 there tells an already-authenticated
//! caller to re-authenticate; a well-behaved client obeys, presents the same valid
//! token, is refused again, and loops — and an interactive client bounces the user
//! through a login form that succeeds and changes nothing.
//!
//! The reverse mistake leaks in the other direction: answering 403 to an
//! *unauthenticated* caller hides the fact that a credential would have helped. So
//! [`forbidden`] refuses to guess. An unauthenticated identity is a caller error
//! and surfaces as a named error, not as a status code — the handler must decide
//! whether to challenge with 401 or to treat the route as anonymous.
//!
//! The body names the required role but never the roles the caller holds. Echoing
//! the held set tells an attacker probing endpoints exactly which role to target.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Build the 403 response for a caller who holds an identity but not the role.
///
/// # Arguments
///
/// * `role` — The role that was required, echoed so an operator can see which gate
///   refused. Safe to echo: it is the server's own literal, not caller input.
///
/// # Returns
///
/// A response map with `status` 403, a `content-type` header, and a plain-text
/// body. The shape and lowercase header names match the response maps `http_serve`
/// already consumes.
pub(super) fn forbidden(role: &str) -> Value {
    let mut headers = HashMap::new();
    headers.insert(
        "content-type".into(),
        Value::Str(Rc::new("text/plain; charset=utf-8".into())),
    );

    let mut response = HashMap::new();
    // 403, not 401: see the module note.
    response.insert("status".into(), Value::Int(403));
    response.insert("headers".into(), Value::Map(Rc::new(RefCell::new(headers))));
    response.insert(
        "body".into(),
        Value::Str(Rc::new(format!("forbidden: requires role `{role}`\n"))),
    );
    Value::Map(Rc::new(RefCell::new(response)))
}
