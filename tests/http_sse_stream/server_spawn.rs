//! One spawn attempt: write the program, launch the binary, read the announcement.
//!
//! Split from [`super`] so retry policy and a single attempt stay separate.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use super::super::script;
use super::Server;

/// Spawn the binary on `port` and wait for its bind announcement.
///
/// # Arguments
///
/// * `port` — Port passed through the `RUST_SSE_ADDR` environment variable, which
///   the test program reads rather than hard-coding a port.
///
/// # Returns
///
/// `Some(server)` once the announcement is seen, `None` when the port was already
/// taken or the process died first. Both are retryable, so neither panics.
pub(super) fn try_start(port: u16) -> Option<Server> {
    let dir = std::env::temp_dir().join(format!("tether_sse_stream_{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("sse_stream.tether");
    std::fs::write(&path, script::source()).ok()?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .env("RUST_SSE_ADDR", port.to_string())
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
