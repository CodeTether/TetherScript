//! Tests for how grant patterns match an address.

use super::scope::Scope;
use super::test_support::patterns;
use super::{grant_all, grant_tcp, require, require_tcp, revoke_all};

#[test]
fn an_exact_host_port_grant_permits_only_that_address() {
    revoke_all();
    grant_tcp(&patterns(&["127.0.0.1:8080"])).unwrap();

    assert!(require_tcp("connect", "127.0.0.1", 8080).is_ok());
    assert!(require_tcp("connect", "127.0.0.1", 9090).is_err());
    assert!(require_tcp("connect", "10.0.0.1", 8080).is_err());
}

#[test]
fn a_host_only_grant_permits_any_port_on_that_host() {
    revoke_all();
    grant_tcp(&patterns(&["example.com"])).unwrap();

    assert!(require_tcp("connect", "example.com", 80).is_ok());
    assert!(require_tcp("connect", "example.com", 443).is_ok());
    assert!(require_tcp("connect", "evil.com", 80).is_err());
}

#[test]
fn host_matching_is_case_insensitive() {
    revoke_all();
    grant_tcp(&patterns(&["Example.COM:80"])).unwrap();

    assert!(require_tcp("connect", "example.com", 80).is_ok());
}

#[test]
fn full_access_grants_both_transports() {
    revoke_all();
    grant_all();

    assert!(require_tcp("connect", "anything", 1).is_ok());
    assert!(require("send_to", "anything", 2).is_ok());
}

#[test]
fn an_invalid_grant_is_rejected_at_grant_time() {
    assert!(Scope::parse("host:70000").is_err());
    assert!(Scope::parse("host:notaport").is_err());
    assert!(Scope::parse(":80").is_err());
    assert!(Scope::parse("host:443").is_ok());
}
