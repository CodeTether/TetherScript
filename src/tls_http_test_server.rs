//! One-shot self-signed HTTPS server used by client verification tests.

use std::io::ErrorKind;
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::tls::{test_identity, TlsAcceptor};

pub(crate) fn spawn() -> (String, JoinHandle<()>) {
    let identity = test_identity::localhost();
    let acceptor = TlsAcceptor::from_pem(&identity.certificate, &identity.private_key).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((tcp, _)) => {
                    let _ = acceptor.accept(tcp);
                    return;
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "TLS client did not connect");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("TLS test listener failed: {error}"),
            }
        }
    });
    (format!("https://localhost:{port}/"), handle)
}

pub(crate) fn assert_verify_error(error: &str) {
    assert!(
        error
            .to_ascii_lowercase()
            .contains("certificate verify failed"),
        "expected certificate verification failure, got: {error}"
    );
}
