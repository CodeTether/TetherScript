//! Verified, in-process TLS transport for HTTPS clients and servers.
//!
//! OpenSSL handles TLS records, certificate-chain validation, hostname
//! validation, and TLS version negotiation. Trust anchors come from the host
//! platform's native certificate store.
//!
//! # Usage
//!
//! ```no_run
//! use tetherscript::tls::TlsConnector;
//!
//! let connector = TlsConnector::new()?;
//! let stream = connector.connect("example.com", 443)?;
//! # drop(stream);
//! # Ok::<(), std::io::Error>(())
//! ```

use std::net::TcpStream;

#[cfg(feature = "openssl-tls")]
use openssl::ssl::SslStream;

#[cfg(feature = "openssl-tls")]
#[path = "tls_client.rs"]
mod client;
#[cfg(not(feature = "openssl-tls"))]
#[path = "tls_disabled.rs"]
mod disabled;
#[cfg(feature = "openssl-tls")]
#[path = "tls_roots.rs"]
mod roots;
#[cfg(feature = "openssl-tls")]
#[path = "tls_server.rs"]
mod server;

#[cfg(all(test, feature = "openssl-tls"))]
#[path = "tls_test_identity.rs"]
pub(crate) mod test_identity;

#[cfg(all(test, feature = "openssl-tls"))]
#[path = "tls_http_test_server.rs"]
pub(crate) mod http_test_server;

#[cfg(all(test, feature = "openssl-tls"))]
#[path = "tls_client_test_support.rs"]
mod client_test_support;

#[cfg(all(test, feature = "openssl-tls"))]
#[path = "tls_client_tests.rs"]
mod client_tests;

#[cfg(all(test, feature = "openssl-tls"))]
#[path = "tls_server_tests.rs"]
mod server_tests;

#[cfg(feature = "openssl-tls")]
pub use client::TlsConnector;
#[cfg(not(feature = "openssl-tls"))]
pub use disabled::TlsConnector;
#[cfg(feature = "openssl-tls")]
pub(crate) use server::TlsAcceptor;

/// A verified OpenSSL stream over TCP.
///
/// # Examples
///
/// ```no_run
/// use tetherscript::tls::{TlsConnector, TlsStream};
///
/// let connector = TlsConnector::new()?;
/// let stream: TlsStream = connector.connect("example.com", 443)?;
/// # drop(stream);
/// # Ok::<(), std::io::Error>(())
/// ```
#[cfg(feature = "openssl-tls")]
pub type TlsStream = SslStream<TcpStream>;
#[cfg(not(feature = "openssl-tls"))]
pub type TlsStream = TcpStream;
