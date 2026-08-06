//! Tests for deny-by-default socket enforcement.
//!
//! These run against a thread-local, so each test revokes first rather than
//! assuming a clean slate.

use super::{grant_tcp, grant_udp, require, require_tcp, revoke_all};

use super::test_support::patterns;

#[test]
fn no_grant_denies_tcp_and_names_the_flag() {
    revoke_all();

    let error = require_tcp("resource.tcp_listen", "127.0.0.1", 8080).unwrap_err();

    assert!(error.contains("--grant-tcp"), "got: {error}");
    assert!(error.contains("TCP"), "got: {error}");
}

#[test]
fn no_grant_denies_udp_and_names_the_flag() {
    revoke_all();

    let error = require("resource.udp_bind", "0.0.0.0", 9000).unwrap_err();

    assert!(error.contains("--grant-udp"), "got: {error}");
}

#[test]
fn tcp_and_udp_grants_are_independent() {
    revoke_all();
    grant_tcp(&patterns(&["*"])).unwrap();

    assert!(require_tcp("connect", "host", 1).is_ok());
    assert!(require("send_to", "host", 1).is_err());
}

#[test]
fn out_of_scope_wording_differs_from_no_grant() {
    revoke_all();
    grant_tcp(&patterns(&["allowed.com:443"])).unwrap();

    let error = require_tcp("connect", "other.com", 443).unwrap_err();

    assert!(error.contains("outside the granted scope"), "got: {error}");
}

#[test]
fn a_star_grant_permits_anything() {
    revoke_all();
    grant_udp(&patterns(&["*"])).unwrap();

    assert!(require("send_to", "8.8.8.8", 53).is_ok());
}
