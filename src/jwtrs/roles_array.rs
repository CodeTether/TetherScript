//! Reading one Keycloak `roles` array.
//!
//! One responsibility: given the `roles` member of a container object, return the
//! role strings. Shared by the realm and per-client paths so the array grammar and
//! its bound are written once.
//!
//! # Why the bound comes before the copy
//!
//! [`MAX_ROLES`] is checked against the array's length *before* any element is
//! cloned, so a payload advertising a million roles is refused without first
//! allocating a million `String`s. Checking after the copy would make the limit a
//! report rather than a defence.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::roles_array::roles_in;
//! use tetherscript::jwtrs::segment::decode_object;
//!
//! // A `realm_access` container, decoded the same way the payload is.
//! let container = decode_object("payload", &encode(br#"{"roles":["admin","user"]}"#)).unwrap();
//! let found = roles_in("realm_access", container.get("roles").cloned()).unwrap();
//! assert_eq!(found, vec!["admin".to_string(), "user".to_string()]);
//!
//! // A container with no `roles` grants nothing rather than failing.
//! let empty = decode_object("payload", &encode(br#"{}"#)).unwrap();
//! assert!(roles_in("realm_access", empty.get("roles").cloned()).unwrap().is_empty());
//! ```

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::limits::MAX_ROLES;
use crate::value::Value;

/// Read a `roles` member's value.
///
/// # Arguments
///
/// * `scope` — Where this array came from, such as `realm_access` or
///   `resource_access.web-app`, used verbatim in error text.
/// * `member` — The `roles` value, or `None` when the container has no `roles`.
///
/// # Returns
///
/// The role names, empty when `member` is `None` or null. A container without
/// `roles` is a token that grants nothing, not a malformed token.
///
/// # Errors
///
/// [`ClaimError::RolesNotArray`] when `roles` is present but not an array,
/// [`ClaimError::TooManyRoles`] past [`MAX_ROLES`], and
/// [`ClaimError::RoleNotString`] when an element is not a string — a numeric or
/// object "role" is refused rather than stringified, since a stringified role could
/// collide with a real role name.
///
/// # Panics
///
/// Does not panic.
pub fn roles_in(scope: &str, member: Option<Value>) -> Result<Vec<String>, ClaimError> {
    let items = match member {
        None | Some(Value::Nil) => return Ok(Vec::new()),
        Some(Value::List(items)) => items,
        Some(other) => {
            return Err(ClaimError::RolesNotArray {
                scope: scope.to_string(),
                found: other.type_name().to_string(),
            });
        }
    };
    let items = items.borrow();
    if items.len() > MAX_ROLES {
        return Err(ClaimError::TooManyRoles {
            scope: scope.to_string(),
            count: items.len(),
            limit: MAX_ROLES,
        });
    }
    items
        .iter()
        .map(|item| match item {
            Value::Str(text) => Ok(text.as_str().to_string()),
            other => Err(ClaimError::RoleNotString {
                scope: scope.to_string(),
                found: other.type_name().to_string(),
            }),
        })
        .collect()
}
