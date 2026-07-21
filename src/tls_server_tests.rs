//! In-process OpenSSL server handshake coverage.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

use super::{test_identity, TlsAcceptor};

#[test]
fn pem_identity_accepts_a_tls_connection() {
    let identity = test_identity::localhost();
    let acceptor = TlsAcceptor::from_pem(&identity.certificate, &identity.private_key).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let mut tls = acceptor.accept(tcp).unwrap();
        tls.write_all(b"secure").unwrap();
    });
    let mut builder = SslConnector::builder(SslMethod::tls_client()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    let tcp = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let mut tls = connector.connect("localhost", tcp).unwrap();
    let mut response = String::new();
    tls.read_to_string(&mut response).unwrap();
    assert_eq!(response, "secure");
    server.join().unwrap();
}
