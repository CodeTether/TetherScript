//! Rejection cases for `--grant-db` connection strings.
//!
//! Split from `db_tests.rs` so each file stays within the line budget, and so the
//! accept and reject paths read separately.

use super::db::parse_url;

/// Extract the rejection message.
///
/// `expect_err` would require `Debug` on `Config`, and `Config` deliberately does
/// not derive it: the struct holds a password, and a panic message must never
/// print one.
fn rejection(url: &str) -> String {
    match parse_url(url) {
        Ok(_) => panic!("`{url}` must be rejected"),
        Err(error) => error,
    }
}

#[test]
fn rejects_a_missing_scheme() {
    assert!(
        rejection("tsuser:pencil@localhost/tsdb").contains("postgres://"),
        "the error should name the required scheme"
    );
}

#[test]
fn rejects_a_missing_database_path() {
    assert!(rejection("postgres://u:p@localhost").contains("/database"));
}

#[test]
fn rejects_a_missing_credentials_separator() {
    assert!(rejection("postgres://localhost/tsdb").contains('@'));
}

#[test]
fn rejects_a_non_numeric_port() {
    assert!(
        rejection("postgres://u:p@localhost:not-a-port/d").contains("port must be a number"),
        "the error should name the offending port"
    );
}

#[test]
fn rejects_an_empty_host() {
    assert!(rejection("postgres://u:p@:5432/d").contains("host"));
}
