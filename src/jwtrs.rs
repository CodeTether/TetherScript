//! RS256 JWT verification against a JWKS provider.
//!
//! Completes the auth chain the port could not reach: the reference application validates
//! Keycloak tokens signed with RS256, while the in-tree JWT only did HS256. That made the
//! port's auth middleware structurally similar but cryptographically different — this
//! closes the gap.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Compact form and segments | `compact`, `segment`, `base64url*` |
//! | Header and payload | `header*`, `payload_fields`, `signature` |
//! | Claims | `claims*`, `audience`, `time_window` |
//! | Keycloak roles | `realm_roles`, `resource_roles`, `roles_array` |
//! | Verification | `verifier`, `validate`, `authenticated`, `config*` |
//! | Bounds and errors | `limits`, `error*` |
//! | Test support | `testdata`, `test_verifier` |
//!
//! # Security posture
//!
//! The signature is verified **before** any claim is read, so a forged token's contents
//! never reach application logic. The algorithm comes from what the verifier accepts, not
//! from the token's own `alg` header — dispatching on the token's claim is the classic
//! algorithm-confusion forgery, and `alg: none` is refused outright. Expiry and
//! not-before are checked with an explicit clock skew rather than an implicit one.

#[path = "jwtrs/audience.rs"]
pub mod audience;
#[path = "jwtrs/authenticated.rs"]
pub mod authenticated;
#[path = "jwtrs/base64url.rs"]
pub mod base64url;
#[path = "jwtrs/base64url_decode.rs"]
pub mod base64url_decode;
#[path = "jwtrs/claims.rs"]
pub mod claims;
#[path = "jwtrs/claims_extract.rs"]
mod claims_extract;
#[path = "jwtrs/compact.rs"]
pub mod compact;
#[path = "jwtrs/config.rs"]
pub mod config;
#[path = "jwtrs/config_build.rs"]
mod config_build;
#[path = "jwtrs/error.rs"]
pub mod error;
#[path = "jwtrs/error_claims.rs"]
pub mod error_claims;
#[path = "jwtrs/error_display.rs"]
mod error_display;
#[path = "jwtrs/error_shape.rs"]
pub mod error_shape;
#[path = "jwtrs/error_text_claims.rs"]
mod error_text_claims;
#[path = "jwtrs/error_text_roles.rs"]
mod error_text_roles;
#[path = "jwtrs/error_text_shape.rs"]
mod error_text_shape;
#[path = "jwtrs/header.rs"]
pub mod header;
#[path = "jwtrs/header_fields.rs"]
mod header_fields;
#[path = "jwtrs/limits.rs"]
pub mod limits;
#[path = "jwtrs/payload_fields.rs"]
mod payload_fields;
#[path = "jwtrs/realm_roles.rs"]
pub mod realm_roles;
#[path = "jwtrs/resource_roles.rs"]
pub mod resource_roles;
#[path = "jwtrs/roles_array.rs"]
pub mod roles_array;
#[path = "jwtrs/segment.rs"]
pub mod segment;
#[path = "jwtrs/signature.rs"]
pub mod signature;
#[path = "jwtrs/test_verifier.rs"]
pub mod test_verifier;
#[path = "jwtrs/testdata.rs"]
pub mod testdata;
#[path = "jwtrs/time_window.rs"]
pub mod time_window;
#[path = "jwtrs/validate.rs"]
pub mod validate;
#[path = "jwtrs/verifier.rs"]
pub mod verifier;
