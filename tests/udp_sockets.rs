//! End-to-end coverage for UDP sockets and socket capability grants.
//!
//! Runs the real binary so the CLI flag plumbing is exercised too, not just the
//! `socket_cap` unit layer. These lock in the fix for sockets reaching the
//! network with no grant under the default `--access-mode restricted`.

use std::process::Command;

/// Run `source` with `flags`, returning `(stdout, stderr, success)`.
fn run(source: &str, flags: &[&str]) -> (String, String, bool) {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_udp_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .args(flags)
        .arg(&path)
        .output()
        .expect("tetherscript should run");

    (
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string(),
        output.status.success(),
    )
}

const BIND: &str =
    "fn main() { let s = resource.udp_bind(\"127.0.0.1\", 0)?  println(s.port()? > 0) }";
const LISTEN: &str =
    "fn main() { let l = resource.tcp_listen(\"127.0.0.1\", 0)?  println(l.port()? > 0) }";

#[test]
fn udp_bind_without_a_grant_is_denied() {
    let (_, stderr, ok) = run(BIND, &[]);

    assert!(!ok, "bind should fail without a grant");
    assert!(stderr.contains("--grant-udp"), "got: {stderr}");
}

#[test]
fn tcp_listen_without_a_grant_is_denied() {
    let (_, stderr, ok) = run(LISTEN, &[]);

    assert!(!ok, "listen should fail without a grant");
    assert!(stderr.contains("--grant-tcp"), "got: {stderr}");
}

#[test]
fn udp_bind_with_a_grant_succeeds() {
    let (stdout, stderr, ok) = run(BIND, &["--grant-udp", "127.0.0.1"]);

    assert!(ok, "bind should succeed with a grant: {stderr}");
    assert_eq!(stdout, "true");
}

#[test]
fn tcp_listen_with_a_grant_succeeds() {
    let (stdout, stderr, ok) = run(LISTEN, &["--grant-tcp", "127.0.0.1"]);

    assert!(ok, "listen should succeed with a grant: {stderr}");
    assert_eq!(stdout, "true");
}

#[test]
fn a_udp_grant_does_not_authorize_tcp() {
    let (_, stderr, ok) = run(LISTEN, &["--grant-udp", "127.0.0.1"]);

    assert!(!ok, "a UDP grant must not authorize TCP");
    assert!(stderr.contains("--grant-tcp"), "got: {stderr}");
}

#[test]
fn an_out_of_scope_port_is_denied() {
    let (stdout, _, ok) = run(
        "fn main() { println(resource.udp_bind(\"127.0.0.1\", 0).is_ok()) }",
        &["--grant-udp", "127.0.0.1:9999"],
    );

    assert!(ok, "the script itself should run");
    assert_eq!(stdout, "false", "port 0 is outside a :9999 grant");
}

#[test]
fn a_malformed_grant_is_rejected_before_the_script_runs() {
    let (_, stderr, ok) = run(BIND, &["--grant-udp", "host:notaport"]);

    assert!(!ok, "a malformed grant should fail");
    assert!(stderr.contains("invalid port"), "got: {stderr}");
}

#[test]
fn a_datagram_round_trips_between_two_sockets() {
    let source = "\
fn main() {
    let server = resource.udp_bind(\"127.0.0.1\", 0)?
    let client = resource.udp_bind(\"127.0.0.1\", 0)?
    client.send_to(\"ping\", \"127.0.0.1\", server.port()?)?
    let mut got = nil
    let mut tries = 0
    while got == nil && tries < 5000 {
        let attempt = server.recv_from(64)
        if attempt.is_ok() { got = attempt.unwrap() }
        tries = tries + 1
    }
    println(got.bytes.decode_utf8())
}";

    let (stdout, stderr, ok) = run(source, &["--grant-udp", "127.0.0.1"]);

    assert!(ok, "round trip should succeed: {stderr}");
    assert_eq!(stdout, "ping");
}
