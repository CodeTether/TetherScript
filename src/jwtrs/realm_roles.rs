//! Keycloak `realm_access.roles` extraction.
//!
//! One responsibility: unwrap the `realm_access` container and delegate to
//! [`roles_in`]. Realm roles apply across every client in the realm, so they are a
//! separate concept from the per-client roles in
//! [`crate::jwtrs::resource_roles`] and get their own file.
//!
//! # Absent is empty, malformed is an error
//!
//! A token with no `realm_access` grants no realm roles, which is normal for a
//! service account. A `realm_access` that is a string or a number is a malformed
//! token, because no issuer produces that; treating it as empty would let a
//! corrupted payload look like a legitimately unprivileged one.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::realm_roles::read;
//! use tetherscript::jwtrs::segment::decode_object;
//!
//! let payload = encode(br#"{"realm_access":{"roles":["offline_access","admin"]}}"#);
//! let members = decode_object("payload", &payload).unwrap();
//! assert_eq!(read(&members).unwrap(), vec!["offline_access".to_string(), "admin".to_string()]);
//!
//! let bare = decode_object("payload", &encode(br#"{}"#)).unwrap();
//! assert!(read(&bare).unwrap().is_empty());
//!
//! let bad = decode_object("payload", &encode(br#"{"realm_access":"admin"}"#)).unwrap();
//! assert!(read(&bad).is_err());
//! ```

use std::collections::HashMap;

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::roles_array::roles_in;
use crate::value::Value;

/// The payload member holding realm-wide roles.
const SCOPE: &str = "realm_access";

/// Extract the realm roles.
///
/// # Arguments
///
/// * `members` — The authenticated payload object.
///
/// # Returns
///
/// The realm role names, empty when `realm_access` is absent or holds no `roles`.
///
/// # Errors
///
/// [`ClaimError::RolesContainerNotObject`] when `realm_access` is present but not an
/// object, plus every error [`roles_in`] can return.
///
/// # Panics
///
/// Does not panic.
pub fn read(members: &HashMap<String, Value>) -> Result<Vec<String>, ClaimError> {
    let container = match members.get(SCOPE) {
        None | Some(Value::Nil) => return Ok(Vec::new()),
        Some(Value::Map(container)) => container.borrow().get("roles").cloned(),
        Some(other) => {
            return Err(ClaimError::RolesContainerNotObject {
                scope: SCOPE.to_string(),
                found: other.type_name().to_string(),
            });
        }
    };
    roles_in(SCOPE, container)
}
