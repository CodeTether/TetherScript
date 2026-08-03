//! Keycloak `resource_access.<client>.roles` extraction.
//!
//! One responsibility: turn the `resource_access` object into a client-keyed role
//! map. Separate from [`crate::jwtrs::realm_roles`] because these roles are scoped
//! to one OAuth client, and conflating the two scopes is an authorization bug: a
//! role named `admin` under client `reports` is not the realm's `admin`.
//!
//! # Determinism
//!
//! Clients are returned in sorted order rather than JSON order. The in-tree parser
//! backs objects with a `HashMap`, whose iteration order is unspecified, so sorting
//! is what makes the result — and therefore any test or audit log built on it —
//! reproducible.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::resource_roles::read;
//! use tetherscript::jwtrs::segment::decode_object;
//!
//! let payload = encode(
//!     br#"{"resource_access":{"web-app":{"roles":["viewer"]},"api":{"roles":["writer"]}}}"#,
//! );
//! let members = decode_object("payload", &payload).unwrap();
//! let clients = read(&members).unwrap();
//! assert_eq!(clients[0].0, "api");
//! assert_eq!(clients[1].1, vec!["viewer".to_string()]);
//! ```

use std::collections::HashMap;

use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::limits::MAX_RESOURCE_CLIENTS;
use crate::jwtrs::roles_array::roles_in;
use crate::value::Value;

/// Extract the per-client roles.
///
/// # Arguments
///
/// * `members` — The authenticated payload object.
///
/// # Returns
///
/// `(client_id, roles)` pairs sorted by client id, empty when `resource_access` is
/// absent.
///
/// # Errors
///
/// [`ClaimError::TooManyResourceClients`] past [`MAX_RESOURCE_CLIENTS`],
/// [`ClaimError::RolesContainerNotObject`] when `resource_access` or one of its
/// values is not an object, plus every error [`roles_in`] can return.
///
/// # Panics
///
/// Does not panic.
pub fn read(members: &HashMap<String, Value>) -> Result<Vec<(String, Vec<String>)>, ClaimError> {
    let clients = match members.get("resource_access") {
        None | Some(Value::Nil) => return Ok(Vec::new()),
        Some(Value::Map(clients)) => clients.borrow().clone(),
        Some(other) => {
            return Err(ClaimError::RolesContainerNotObject {
                scope: "resource_access".to_string(),
                found: other.type_name().to_string(),
            });
        }
    };
    if clients.len() > MAX_RESOURCE_CLIENTS {
        return Err(ClaimError::TooManyResourceClients {
            count: clients.len(),
            limit: MAX_RESOURCE_CLIENTS,
        });
    }
    let mut names: Vec<&String> = clients.keys().collect();
    names.sort();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let scope = format!("resource_access.{name}");
        let member = match clients.get(name) {
            Some(Value::Map(entry)) => entry.borrow().get("roles").cloned(),
            Some(other) => {
                return Err(ClaimError::RolesContainerNotObject {
                    scope,
                    found: other.type_name().to_string(),
                });
            }
            None => None,
        };
        out.push((name.clone(), roles_in(&scope, member)?));
    }
    Ok(out)
}
