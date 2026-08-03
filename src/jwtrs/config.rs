//! Validation policy: what a token must say to be accepted here.
//!
//! One responsibility: hold the *verifier's* expectations as data. The
//! constructors and builder methods live in [`crate::jwtrs::config_build`] so this
//! file is only the shape.
//!
//! # Why the algorithm lives in the config
//!
//! This is the whole point of the type. The header's `alg` is attacker-chosen, so
//! it can only ever be *compared* against [`ValidationConfig::algorithm`], never
//! dispatched on. Storing the algorithm here, next to the issuer and audience,
//! makes it obvious that it is deployment policy rather than token data. See
//! [`crate::jwtrs::header`] for the comparison itself.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::config::ValidationConfig;
//!
//! let config = ValidationConfig::rs256("https://sso.example/realms/main", ["web-app"]);
//! assert_eq!(config.algorithm, "RS256");
//! assert_eq!(config.skew_secs, 60);
//! assert_eq!(config.audiences, vec!["web-app".to_string()]);
//! ```

/// The expectations a token is checked against.
///
/// Construct with [`ValidationConfig::rs256`]; there is deliberately no
/// `Default`, because a config with an empty issuer is never what anyone wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationConfig {
    /// The exact `iss` required, compared byte for byte against the claim.
    pub issuer: String,
    /// Accepted `aud` values. A token matches when *any* of its audiences is here.
    pub audiences: Vec<String>,
    /// The one algorithm accepted. Compared against the header, never dispatched on.
    pub algorithm: &'static str,
    /// Symmetric clock tolerance in seconds; see
    /// [`DEFAULT_SKEW_SECS`](crate::jwtrs::limits::DEFAULT_SKEW_SECS).
    pub skew_secs: i64,
    /// Required header `typ`, or `None` to accept any. Keycloak sends `"JWT"`.
    pub required_typ: Option<String>,
}
