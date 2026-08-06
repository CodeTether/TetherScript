//! `SocketAuthority` — TCP and UDP socket access as a capability.
//!
//! Before this existed, `resource.tcp_listen` and `resource.tcp_connect` reached
//! the network with no grant at all: a script running under the default
//! `--access-mode restricted` could bind or dial any address. That was the same
//! ambient-authority gap the `fs_*` built-ins have, and it silently contradicted
//! the posture the rest of the capability system enforces.
//!
//! Sockets now require `--grant-tcp` / `--grant-udp`, or `--access-mode full`.
//!
//! # Scope
//!
//! A grant is a list of `host`, `host:port`, or `*` patterns. TCP and UDP are
//! granted separately, because listening for datagrams and dialing a TCP service
//! are different authorities.
//!
//! # Why a thread-local check
//!
//! Owned socket resources are constructed deep inside `value::resource`, which
//! has no capability handle threaded through it. Rather than restructure every
//! factory signature inside a security fix, the grant is consulted at the syscall
//! boundary. The check is *deny by default*: with no grant installed, [`require`]
//! and [`require_tcp`] both fail.

mod check;
mod grants;
pub mod scope;
mod transport;

pub use grants::{grant_all, grant_tcp, grant_udp};

/// Revoke every socket grant on this thread.
///
/// Exposed for embedders and tests that reset authority between runs. The CLI
/// installs grants once per process and never revokes, so the binary target does
/// not reference this.
#[cfg_attr(not(test), allow(unused_imports))]
pub use grants::revoke_all;
pub use transport::Transport;

/// Authorize a TCP operation against `host:port`.
///
/// # Errors
///
/// Returns a message naming the operation, the address, and the flag that would
/// grant it when no installed scope permits the address.
pub fn require_tcp(operation: &str, host: &str, port: u16) -> Result<(), String> {
    check::run(Transport::Tcp, operation, host, port)
}

/// Authorize a UDP operation against `host:port`.
///
/// # Errors
///
/// As [`require_tcp`], for the UDP grant.
pub fn require(operation: &str, host: &str, port: u16) -> Result<(), String> {
    check::run(Transport::Udp, operation, host, port)
}

#[cfg(test)]
mod scope_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
