//! Blocking HTTP/1.1 and verified HTTPS clients and servers.
//!
//! This module re-exports the public API from focused sub-modules:
//!
//! - [`get`], [`head`], [`post`], [`request`] - blocking client functions
//! - [`serve`] - blocking server accept loop
//! - `http://` and `https://` URL parsing used by client helpers
//!
//! Internal sub-modules handle OpenSSL transport, bounded request parsing,
//! headers, response serialization, and status reason phrases.

#[path = "http_client.rs"]
mod http_client;
#[path = "http_headers.rs"]
mod http_headers;
#[path = "http_response.rs"]
mod http_response;
#[path = "http_response_extract.rs"]
mod http_response_extract;
#[path = "http_server.rs"]
mod http_server;
#[path = "http_server_args.rs"]
mod http_server_args;
#[path = "http_server_connection.rs"]
mod http_server_connection;
#[path = "http_server_headers.rs"]
mod http_server_headers;
#[path = "http_server_reader.rs"]
mod http_server_reader;
#[path = "http_server_request.rs"]
mod http_server_request;
#[path = "http_server_request_map.rs"]
mod http_server_request_map;
#[path = "http_static/mod.rs"]
mod http_static;
#[path = "http_status.rs"]
mod http_status;
#[path = "http_stream.rs"]
mod http_stream;
#[path = "http_url.rs"]
mod http_url;
#[path = "https_server.rs"]
mod https_server;

pub(crate) use http_client::client_request;
pub use http_client::{get, head, post, request};
pub use http_server::serve;
pub(crate) use http_static::serve as serve_static;
pub(crate) use https_server::serve as serve_tls;

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tls_http_tests.rs"]
mod tls_http_tests;
#[cfg(test)]
#[path = "tls_https_server_vm_tests.rs"]
mod tls_https_server_vm_tests;
