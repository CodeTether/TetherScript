//! # RS256 JWT claims
//!
//! One responsibility: own the [`Claims`] type — the typed, already-validated
//! result of RS256 token validation — and the extraction that fills it from an
//! [`Authenticated`] payload.
//!
//! ## What a `Claims` value means
//!
//! Holding one is a statement with teeth: the signature was verified by a
//! [`SignatureVerifier`](crate::jwtrs::verifier::SignatureVerifier), the header's
//! `alg` equalled the configured algorithm, `iss` matched, `aud` intersected the
//! accepted audiences, and `now` was inside `[nbf - skew, exp + skew)`. There is no
//! constructor that skips any of that: the only public way in is
//! [`Claims::validate`], and the crate-internal extraction it calls requires an
//! [`Authenticated`], which in turn cannot exist without a successful signature
//! check. See [`crate::jwtrs::authenticated`] for why that ordering is structural.
//!
//! ## What is deliberately absent
//!
//! No RSA arithmetic, no JWKS parsing, no HTTP, and no clock. This is the
//! claim-validation half of RS256 support and stops precisely there.
//!
//! ## Security note on `kid`
//!
//! [`Claims::kid`] is carried for audit logging only. It arrived in the
//! *unverified* header and is attacker-chosen. It selects a key by equality against
//! published identifiers and must never be interpolated into a filesystem path, a
//! URL, or a query — a `kid` of `../../etc/shadow` would otherwise turn key
//! selection into arbitrary file read.
//!
//! ## Examples
//!
//! ```rust
//! use tetherscript::jwtrs::claims::Claims;
//! use tetherscript::jwtrs::config::ValidationConfig;
//! use tetherscript::jwtrs::test_verifier::StubVerifier;
//! use tetherscript::jwtrs::testdata::keycloak_token;
//!
//! let config = ValidationConfig::rs256("https://sso.example/realms/main", ["web-app"]);
//! let claims = Claims::validate(
//!     &keycloak_token(1_000, "sig-ok"),
//!     &config,
//!     950,
//!     &StubVerifier::accepting("sig-ok"),
//! )
//! .unwrap();
//!
//! assert_eq!(claims.sub, "user-1");
//! assert_eq!(claims.azp.as_deref(), Some("web-app"));
//! assert!(claims.has_realm_role("admin"));
//! assert!(claims.has_resource_role("web-app", "viewer"));
//! ```

use crate::jwtrs::authenticated::Authenticated;
use crate::jwtrs::error_claims::ClaimError;

/// A validated RS256 token's claims.
///
/// Every field is populated from a payload whose signature was already accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claims {
    /// `iss`, already confirmed equal to the configured issuer.
    pub iss: String,
    /// `sub`, the subject identifier. Required: an unattributable token is useless.
    pub sub: String,
    /// `aud`, normalised to a list, already confirmed to intersect the config.
    pub aud: Vec<String>,
    /// `exp`, in seconds since the Unix epoch. Never absent; see
    /// [`crate::jwtrs::time_window`].
    pub exp: i64,
    /// `nbf`, when the issuer supplied one.
    pub nbf: Option<i64>,
    /// `iat`, informational only; it never gates acceptance.
    pub iat: Option<i64>,
    /// `azp`, Keycloak's authorized-party client id.
    pub azp: Option<String>,
    /// `realm_access.roles`, in issuer order.
    pub realm_roles: Vec<String>,
    /// `resource_access.<client>.roles`, sorted by client id.
    pub resource_roles: Vec<(String, Vec<String>)>,
    /// The header's `kid`. **Attacker-controlled**; for audit logging only.
    pub kid: Option<String>,
}

impl Claims {
    /// Report whether a realm-wide role was granted.
    ///
    /// # Arguments
    ///
    /// * `role` — The role name, compared byte-exactly.
    ///
    /// # Returns
    ///
    /// `true` when `realm_access.roles` contains `role`.
    pub fn has_realm_role(&self, role: &str) -> bool {
        self.realm_roles.iter().any(|held| held == role)
    }

    /// Report whether a client-scoped role was granted.
    ///
    /// Realm and resource roles are kept apart on purpose: a `reports` client's
    /// `admin` is not the realm's `admin`, and merging them would silently widen
    /// authority.
    ///
    /// # Arguments
    ///
    /// * `client` — The OAuth client id, as it appears under `resource_access`.
    /// * `role` — The role name.
    ///
    /// # Returns
    ///
    /// `true` when that client grants that role.
    pub fn has_resource_role(&self, client: &str, role: &str) -> bool {
        self.resource_roles
            .iter()
            .any(|(id, roles)| id == client && roles.iter().any(|held| held == role))
    }

    /// Pull the claim set out of an already-authenticated payload.
    ///
    /// Callers normally want [`Claims::validate`], which also enforces issuer,
    /// audience, and the time window. This function performs extraction only, so it
    /// is `pub(crate)`: returning a `Claims` that has not been through those checks
    /// would make the type's guarantee a lie.
    ///
    /// # Arguments
    ///
    /// * `token` — A payload whose signature a verifier accepted.
    ///
    /// # Returns
    ///
    /// The populated claim set.
    ///
    /// # Errors
    ///
    /// Every [`ClaimError`] arising from a missing or wrong-typed claim.
    pub(crate) fn extract(token: &Authenticated) -> Result<Self, ClaimError> {
        crate::jwtrs::claims_extract::extract(token)
    }
}
