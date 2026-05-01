//! Zero-Rust-dependency TLS stream used by ProviderAuthority.
//!
//! This module intentionally avoids Rust TLS crates. It delegates TLS record
//! handling and certificate verification to the platform OpenSSL executable
//! (`openssl s_client`). That keeps Cargo dependency count at zero while still
//! using the system TLS implementation and CA bundle.
//!
//! The public surface is intentionally tiny: enough for HTTPS POST over an
//! already-connected TCP target.

use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct TlsConnector;

impl TlsConnector {
    pub fn new() -> io::Result<Self> {
        Ok(Self)
    }

    pub fn connect(&self, domain: &str, port: u16) -> io::Result<TlsStream> {
        TlsStream::connect(domain, port)
    }
}

pub struct TlsStream {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl TlsStream {
    fn connect(domain: &str, port: u16) -> io::Result<Self> {
        let connect = format!("{}:{}", domain, port);
        let mut child = Command::new("openssl")
            .arg("s_client")
            .arg("-quiet")
            .arg("-servername")
            .arg(domain)
            .arg("-connect")
            .arg(connect)
            .arg("-verify_return_error")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!(
                        "failed to spawn `openssl s_client`; install OpenSSL or use http:// reverse proxy: {}",
                        e
                    ),
                )
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("openssl stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("openssl stdout unavailable"))?;

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush()
    }
}

impl Drop for TlsStream {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
