//! Spawning the real binary and waiting for its own bind announcement.
//!
//! Binding a port to discover a free one and then releasing it is inherently
//! racy, so [`start`] retries. Waiting for the server's *own* stderr line — rather
//! than sleeping — is what makes the suite deterministic instead of flaky under
//! load.

use std::net::TcpListener;
use std::process::Child;

#[path = "server_spawn.rs"]
mod spawn;

/// A running server, killed on drop so a failed assertion cannot leak a process.
pub(crate) struct Server {
    /// The child process. Held solely to kill it on drop.
    pub(super) child: Child,
    /// Port the server announced.
    pub(crate) port: u16,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Probe for a likely-free port by binding and immediately releasing one.
fn candidate_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral")
        .local_addr()
        .expect("addr")
        .port()
}

/// Start the server, retrying on port collisions.
///
/// # Returns
///
/// A live [`Server`].
///
/// # Panics
///
/// Panics after eight failed attempts, which means something other than a port
/// race is wrong — a missing binary, or a program that fails to parse.
pub(crate) fn start() -> Server {
    for _ in 0..8 {
        if let Some(server) = spawn::try_start(candidate_port()) {
            return server;
        }
    }
    panic!("server did not start after 8 attempts");
}
