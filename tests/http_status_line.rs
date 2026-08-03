//! Status-line correctness observed over the wire.
//!
//! Unmapped statuses previously fell back to `OK`, so a `429` went out as
//! `HTTP/1.1 429 OK` — a status line that contradicts itself and that a strict client or
//! proxy may reject. The phrases are asserted through a real socket rather than by calling
//! the private helper, so the test covers what a client actually receives.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};

/// A running server, killed on drop.
struct Server {
    child: Child,
    port: u16,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Probe for a likely-free port.
fn candidate_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral")
        .local_addr()
        .expect("addr")
        .port()
}

/// Serve one handler that echoes the status from the path, e.g. `/429`.
fn start() -> Server {
    let source = "fn main() {\n\
         http_serve(port(), fn(req) {\n\
             let code = parse_int(req.path.replace(\"/\", \"\"))\n\
             let resp = map()\n\
             resp.status = code.unwrap()\n\
             resp.body = \"x\"\n\
             resp.headers = map()\n\
             resp\n\
         })\n\
         }\n\
         fn port() {\n\
             return parse_int(env_get(\"RUST_HTTP_ADDR\").unwrap()).unwrap()\n\
         }\n";
    for _ in 0..8 {
        let port = candidate_port();
        if let Some(server) = try_start(source, port) {
            return server;
        }
    }
    panic!("server did not start after 8 attempts");
}

/// Spawn the server and wait for its own bind announcement.
fn try_start(source: &str, port: u16) -> Option<Server> {
    let dir = std::env::temp_dir().join(format!("tether_status_{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("status.tether");
    std::fs::write(&path, source).ok()?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .env("RUST_HTTP_ADDR", port.to_string())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    let stderr = child.stderr.take()?;
    let expected = format!("listening on http://0.0.0.0:{port}");
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return None,
            Ok(_) if line.contains(&expected) => return Some(Server { child, port }),
            Ok(_) if line.contains("Address already in use") => return None,
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

/// Read the status line for a request to `path`.
fn status_line(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .expect("send");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read");
    response.lines().next().unwrap_or_default().to_string()
}

/// The case that motivated this: a throttled response must not claim success.
#[test]
fn a_429_does_not_claim_ok() {
    let server = start();
    assert_eq!(
        status_line(server.port, "/429"),
        "HTTP/1.1 429 Too Many Requests"
    );
}

/// 410 is how the shortener reports an expired code, so it is on a live path.
#[test]
fn a_410_reports_gone() {
    let server = start();
    assert_eq!(status_line(server.port, "/410"), "HTTP/1.1 410 Gone");
}

#[test]
fn a_409_reports_conflict() {
    let server = start();
    assert_eq!(status_line(server.port, "/409"), "HTTP/1.1 409 Conflict");
}

#[test]
fn common_statuses_keep_their_phrases() {
    let server = start();
    assert_eq!(status_line(server.port, "/200"), "HTTP/1.1 200 OK");
    assert_eq!(status_line(server.port, "/404"), "HTTP/1.1 404 Not Found");
    assert_eq!(
        status_line(server.port, "/500"),
        "HTTP/1.1 500 Internal Server Error"
    );
}

/// An unregistered code must be neutral, never `OK`.
#[test]
fn an_unmapped_status_is_unknown_not_ok() {
    let server = start();
    let line = status_line(server.port, "/599");
    assert!(!line.contains("OK"), "must not claim success: {line}");
    assert_eq!(line, "HTTP/1.1 599 Unknown");
}
