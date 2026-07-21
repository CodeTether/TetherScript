//! Deterministic certificate-chain and hostname verification tests.

use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use super::{test_identity, TlsAcceptor, TlsConnector};

fn spawn(identity: &test_identity::TestIdentity) -> (u16, JoinHandle<()>) {
    let acceptor = TlsAcceptor::from_pem(&identity.certificate, &identity.private_key).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let _ = acceptor.accept(tcp);
    });
    (port, server)
}

#[test]
fn trusted_chain_and_matching_hostname_connect() {
    let identity = test_identity::localhost();
    let connector = TlsConnector::trusting(&identity.certificate).unwrap();
    let (port, server) = spawn(&identity);
    let tcp = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let stream = connector.connect_tcp("localhost", tcp).unwrap();
    assert_eq!(stream.ssl().verify_result().error_string(), "ok");
    drop(stream);
    server.join().unwrap();
}

#[test]
fn trusted_chain_with_wrong_hostname_is_rejected() {
    let identity = test_identity::localhost();
    let connector = TlsConnector::trusting(&identity.certificate).unwrap();
    let (port, server) = spawn(&identity);
    let tcp = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let error = connector
        .connect_tcp("wrong.example", tcp)
        .expect_err("hostname mismatch should fail");
    assert!(
        error
            .to_string()
            .to_ascii_lowercase()
            .contains("certificate verify failed"),
        "expected hostname verification failure, got: {error}"
    );
    server.join().unwrap();
}
