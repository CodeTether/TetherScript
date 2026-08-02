//! `sslmode` parsing for `--grant-db`.
//!
//! Every rejection here is deliberate: a mode that silently produced cleartext
//! would leave a caller believing the connection was encrypted.

use super::db::parse_url;

/// Whether a connection string requests TLS.
fn tls_for(url: &str) -> bool {
    parse_url(url).expect("url should parse").tls
}

/// Extract the rejection message.
fn rejection(url: &str) -> String {
    match parse_url(url) {
        Ok(_) => panic!("`{url}` must be rejected"),
        Err(error) => error,
    }
}

#[test]
fn tls_is_off_by_default() {
    assert!(!tls_for("postgres://u:p@localhost/app"));
}

#[test]
fn require_and_verify_full_enable_tls() {
    assert!(tls_for("postgres://u:p@localhost/app?sslmode=require"));
    assert!(tls_for("postgres://u:p@localhost/app?sslmode=verify-full"));
}

#[test]
fn disable_keeps_cleartext() {
    assert!(!tls_for("postgres://u:p@localhost/app?sslmode=disable"));
}

/// The database name must not absorb the query string.
#[test]
fn the_query_string_is_not_part_of_the_database_name() {
    let config = parse_url("postgres://u:p@localhost/app?sslmode=require").expect("parse");
    assert_eq!(config.database, "app");
}

/// `prefer` falls back to cleartext without telling anyone, so it is refused.
#[test]
fn prefer_is_refused_rather_than_downgraded() {
    let error = rejection("postgres://u:p@localhost/app?sslmode=prefer");
    assert!(error.contains("cleartext"), "got: {error}");
}

#[test]
fn verify_ca_is_refused_in_favour_of_verify_full() {
    let error = rejection("postgres://u:p@localhost/app?sslmode=verify-ca");
    assert!(error.contains("verify-full"), "got: {error}");
}

#[test]
fn an_unknown_sslmode_names_the_supported_values() {
    let error = rejection("postgres://u:p@localhost/app?sslmode=maybe");
    assert!(error.contains("require"), "got: {error}");
}

#[test]
fn an_unknown_query_parameter_is_refused() {
    let error = rejection("postgres://u:p@localhost/app?timeout=5");
    assert!(error.contains("timeout"), "got: {error}");
}
