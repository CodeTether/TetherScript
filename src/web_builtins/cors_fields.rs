//! Key names for the CORS config and policy maps.
//!
//! Centralized so the validator, the reader, and the response builders cannot
//! drift: a policy written under one spelling and read under another would look
//! like an empty allow-list, which fails closed but is baffling to debug.

/// Config/policy key holding the allow-list of origins, or the string `"*"`.
pub(super) const ORIGINS: &str = "origins";
/// Config/policy key holding the allowed request methods.
pub(super) const METHODS: &str = "methods";
/// Config/policy key holding the allowed request header names.
pub(super) const HEADERS: &str = "headers";
/// Config/policy key holding the response headers exposed to script.
pub(super) const EXPOSE: &str = "expose";
/// Config/policy key holding the credentials flag.
pub(super) const CREDENTIALS: &str = "credentials";
/// Config/policy key holding the preflight cache lifetime in seconds.
pub(super) const MAX_AGE: &str = "max_age";
/// Policy-only key recording that `origins` was the wildcard.
pub(super) const WILDCARD: &str = "wildcard";

/// Every key `cors_policy` accepts.
///
/// An unknown key is an error rather than being ignored: `origin` instead of
/// `origins` would otherwise produce a policy that allows nothing, and the
/// symptom (every cross-origin request quietly failing) points nowhere near the
/// typo.
pub(super) const CONFIG_KEYS: [&str; 6] = [ORIGINS, METHODS, HEADERS, EXPOSE, CREDENTIALS, MAX_AGE];

/// Request header a preflight uses to announce the real method.
pub(super) const REQUEST_METHOD: &str = "access-control-request-method";
/// Request header a preflight uses to announce the real headers.
pub(super) const REQUEST_HEADERS: &str = "access-control-request-headers";
/// The `Origin` request header.
pub(super) const ORIGIN: &str = "origin";
