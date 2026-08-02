//! Connection-string parsing for `--grant-db`.
//!
//! Parsing is tested without a server because a malformed URL must be rejected
//! before any connection is attempted; a wrong default database would otherwise
//! surface as a confusing query error much later.

use super::db::parse_url;

#[test]
fn parses_a_full_connection_string() {
    let config = parse_url("postgres://tsuser:pencil@127.0.0.1:55432/tsdb").expect("parse");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 55432);
    assert_eq!(config.user, "tsuser");
    assert_eq!(config.password, "pencil");
    assert_eq!(config.database, "tsdb");
}

#[test]
fn defaults_the_port_when_absent() {
    let config = parse_url("postgres://u:p@db.internal/app").expect("parse");
    assert_eq!(config.host, "db.internal");
    assert_eq!(config.port, 5432);
}

#[test]
fn accepts_the_postgresql_scheme_alias() {
    assert!(parse_url("postgresql://u:p@h/d").is_ok());
}

/// A passwordless role is legitimate for trust authentication.
#[test]
fn accepts_a_user_with_no_password() {
    let config = parse_url("postgres://tsuser@localhost/tsdb").expect("parse");
    assert_eq!(config.user, "tsuser");
    assert_eq!(config.password, "");
}
