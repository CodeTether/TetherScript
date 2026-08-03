//! Constructors and builder methods for [`ValidationConfig`].
//!
//! One responsibility: the safe ways to build a config. Split from
//! [`crate::jwtrs::config`] so the data shape and the construction policy are
//! separate reads.
//!
//! # Why `rs256` and not a general constructor
//!
//! Naming the algorithm in the *constructor* means a caller cannot forget to pin
//! one. There is no `ValidationConfig::new(alg)` taking a caller-supplied string,
//! because the next step after that would be someone passing the token's own
//! `alg` into it, which is precisely the algorithm-confusion attack.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::config::ValidationConfig;
//!
//! let config = ValidationConfig::rs256("https://sso.example/realms/main", ["web-app", "api"])
//!     .with_skew_secs(5)
//!     .requiring_typ("JWT");
//! assert_eq!(config.audiences.len(), 2);
//! assert_eq!(config.skew_secs, 5);
//! assert_eq!(config.required_typ.as_deref(), Some("JWT"));
//! ```

use crate::jwtrs::config::ValidationConfig;
use crate::jwtrs::limits::DEFAULT_SKEW_SECS;

impl ValidationConfig {
    /// Build a config pinned to `RS256`, with the default clock skew.
    ///
    /// # Arguments
    ///
    /// * `issuer` — The exact expected `iss`.
    /// * `audiences` — Accepted `aud` values.
    ///
    /// # Returns
    ///
    /// A config with `algorithm` fixed to `"RS256"`, `skew_secs` set to
    /// [`DEFAULT_SKEW_SECS`], and no `typ` requirement.
    ///
    /// # Panics
    ///
    /// Does not panic. An empty `audiences` list is accepted and makes every token
    /// fail the audience check, which is the safe direction to fail.
    pub fn rs256<I, S>(issuer: &str, audiences: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            issuer: issuer.to_string(),
            audiences: audiences.into_iter().map(Into::into).collect(),
            algorithm: "RS256",
            skew_secs: DEFAULT_SKEW_SECS,
            required_typ: None,
        }
    }

    /// Override the symmetric clock-skew tolerance.
    ///
    /// # Arguments
    ///
    /// * `seconds` — Tolerance in seconds. A negative value is clamped to `0`,
    ///   because a negative skew would *shorten* the window on one side and widen
    ///   it on the other, which no caller means.
    ///
    /// # Returns
    ///
    /// The updated config.
    pub fn with_skew_secs(mut self, seconds: i64) -> Self {
        self.skew_secs = seconds.max(0);
        self
    }

    /// Require an exact header `typ`.
    ///
    /// # Arguments
    ///
    /// * `typ` — The required media type, normally `"JWT"`.
    ///
    /// # Returns
    ///
    /// The updated config.
    pub fn requiring_typ(mut self, typ: &str) -> Self {
        self.required_typ = Some(typ.to_string());
        self
    }
}
