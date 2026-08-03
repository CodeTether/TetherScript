//! The RFC 6455 handshake: the published key/accept vector and header rules.

use tetherscript::websocket::accept::accept_key;
use tetherscript::websocket::handshake::validate_request;
use tetherscript::websocket::handshake_error::HandshakeError;
use tetherscript::websocket::response::switching_protocols;

/// Look up a header case-insensitively from a slice of pairs.
fn lookup<'a>(headers: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
}

/// The complete, valid request from RFC 6455 §1.3.
fn valid() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Upgrade", "websocket"),
        ("Connection", "keep-alive, Upgrade"),
        ("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="),
        ("Sec-WebSocket-Version", "13"),
    ]
}

#[test]
fn rfc6455_example_key_produces_the_published_accept_value() {
    assert_eq!(
        accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
}

#[test]
fn a_valid_request_yields_the_accept_value_and_a_101_response() {
    let headers = valid();
    let accept = validate_request(|name| lookup(&headers, name)).expect("handshake is valid");
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    let response = switching_protocols(&accept);
    assert!(response.starts_with("HTTP/1.1 101 Switching Protocols\r\n"));
    assert!(response.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"));
    assert!(response.ends_with("\r\n\r\n"));
}

#[test]
fn a_missing_required_header_is_rejected() {
    for drop in ["Upgrade", "Connection", "Sec-WebSocket-Version"] {
        let headers: Vec<_> = valid().into_iter().filter(|(k, _)| *k != drop).collect();
        let error = validate_request(|name| lookup(&headers, name)).expect_err("must reject");
        assert!(matches!(error, HandshakeError::MissingHeader { .. }));
    }
}
