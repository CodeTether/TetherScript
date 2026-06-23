//! Vault body decoder tests.

use super::body;

#[test]
fn decodes_chunked_response_body() {
    let head = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked";
    let body = b"5\r\nhello\r\n1\r\n!\r\n0\r\n\r\n";
    assert_eq!(body::decode(head, body).unwrap(), b"hello!");
}

#[test]
fn leaves_plain_response_body_unchanged() {
    let head = "HTTP/1.1 200 OK\r\nContent-Length: 5";
    assert_eq!(body::decode(head, b"hello").unwrap(), b"hello");
}
