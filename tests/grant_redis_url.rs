//! URL parsing for the `--grant-redis` CLI flag.
//!
//! Every case here runs without a Redis server, which is the point: a malformed URL
//! must be rejected *before* a socket is opened, and `rediss://` must be rejected
//! before a password crosses the network in cleartext.
//!
//! Password secrecy is a separate invariant with its own file,
//! `tests/grant_redis_password_secrecy.rs`.
//!
//! # Why these assert on message substrings
//!
//! [`Config`](tetherscript::redis::Config) deliberately does not derive `Debug`,
//! because it holds a password and a panic message must never print one. So
//! `expect_err` is unavailable, and these tests match on the message text instead —
//! which is what the caller actually sees, so it is the more honest assertion.

use tetherscript::redis_cap::url;

/// Extract the rejection message, without requiring `Debug` on `Config`.
fn rejection(target: &str) -> String {
    match url::parse(target) {
        Ok(_) => panic!("`{target}` must be rejected"),
        Err(error) => error,
    }
}

#[test]
fn parses_a_full_url() {
    let config = url::parse("redis://app:pencil@cache.internal:6380/3").expect("parse");
    assert_eq!(config.host, "cache.internal");
    assert_eq!(config.port, 6380);
    assert_eq!(config.username.as_deref(), Some("app"));
    assert_eq!(config.password.as_deref(), Some("pencil"));
    assert_eq!(config.database, 3);
}

#[test]
fn a_bare_host_defaults_the_port_and_database() {
    let config = url::parse("redis://localhost").expect("parse");
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 6379);
    assert_eq!(config.database, 0);
    assert!(config.username.is_none());
    assert!(config.password.is_none(), "no AUTH without a password");
}

#[test]
fn accepts_a_host_and_port_with_no_path() {
    let config = url::parse("redis://127.0.0.1:6380").expect("parse");
    assert_eq!(config.port, 6380);
    assert_eq!(config.database, 0);
}

#[test]
fn accepts_a_host_and_database_with_the_default_port() {
    let config = url::parse("redis://cache/9").expect("parse");
    assert_eq!(config.port, 6379);
    assert_eq!(config.database, 9);
}

/// `redis://host/` and `redis://host` both mean database 0, as Redis defaults.
#[test]
fn an_empty_path_is_database_zero() {
    assert_eq!(url::parse("redis://cache/").expect("parse").database, 0);
}

/// The password-only form drives the legacy one-argument `AUTH`.
#[test]
fn accepts_a_password_with_no_username() {
    let config = url::parse("redis://:pencil@cache").expect("parse");
    assert!(config.username.is_none());
    assert_eq!(config.password.as_deref(), Some("pencil"));
}

/// An ACL username with no password sends no `AUTH`, since `Config` skips it.
#[test]
fn accepts_a_username_with_no_password() {
    let config = url::parse("redis://app@cache").expect("parse");
    assert_eq!(config.username.as_deref(), Some("app"));
    assert!(config.password.is_none());
}

/// `requirepass ""` is a real configuration, distinct from having no password at all.
#[test]
fn an_empty_password_is_not_the_same_as_no_password() {
    let config = url::parse("redis://app:@cache").expect("parse");
    assert_eq!(config.password.as_deref(), Some(""));
}

/// A password containing `@` splits on the *last* `@`, so the host is still right.
///
/// Percent-decoding is deliberately absent, but the split must not be ambiguous.
#[test]
fn a_password_containing_an_at_sign_still_finds_the_host() {
    let config = url::parse("redis://app:pa@ss@cache.internal:6380/1").expect("parse");
    assert_eq!(config.host, "cache.internal");
    assert_eq!(config.port, 6380);
    assert_eq!(config.password.as_deref(), Some("pa@ss"));
}

/// A password containing `:` keeps everything after the first colon.
#[test]
fn a_password_containing_a_colon_is_kept_whole() {
    let config = url::parse("redis://app:pa:ss@cache").expect("parse");
    assert_eq!(config.username.as_deref(), Some("app"));
    assert_eq!(config.password.as_deref(), Some("pa:ss"));
}

#[test]
fn rejects_a_missing_scheme() {
    assert!(
        rejection("localhost:6379/0").contains("redis://"),
        "the error should name the required scheme"
    );
}

#[test]
fn rejects_an_unknown_scheme() {
    assert!(rejection("http://cache/0").contains("redis://"));
}

/// The central security assertion: TLS is refused, not silently downgraded.
#[test]
fn rejects_rediss_because_tls_is_not_wired() {
    let error = rejection("rediss://app:pencil@cache.internal:6380/0");
    assert!(error.contains("rediss://"), "got: {error}");
    assert!(error.contains("TLS"), "the error should name TLS: {error}");
    assert!(
        error.contains("cleartext"),
        "the error should say why it refuses rather than connecting: {error}"
    );
}

#[test]
fn rejects_a_non_numeric_port() {
    let error = rejection("redis://cache:not-a-port/0");
    assert!(error.contains("port must be a number"), "got: {error}");
    assert!(
        error.contains("not-a-port"),
        "the error should name the offending text: {error}"
    );
}

/// A port above 65535 is out of range, not a silently truncated one.
#[test]
fn rejects_an_out_of_range_port() {
    assert!(rejection("redis://cache:70000/0").contains("port"));
}

#[test]
fn rejects_a_missing_host() {
    assert!(rejection("redis://:6379/0").contains("host"));
}

#[test]
fn rejects_a_missing_host_after_credentials() {
    assert!(rejection("redis://app:pencil@/0").contains("host"));
}

#[test]
fn rejects_a_non_numeric_database() {
    let error = rejection("redis://cache/sessions");
    assert!(error.contains("database"), "got: {error}");
    assert!(
        error.contains("sessions"),
        "the error should name the offending text: {error}"
    );
}

#[test]
fn rejects_a_negative_database() {
    assert!(rejection("redis://cache/-1").contains("database"));
}

/// A trailing path segment is refused rather than partially parsed.
#[test]
fn rejects_a_trailing_path_segment() {
    assert!(rejection("redis://cache/0/extra").contains("database"));
}
