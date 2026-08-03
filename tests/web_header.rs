//! Coverage for the header built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: every concern module is private, and the script
//! surface is what a real application actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_header_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("header_case_{case}.tether"));
    std::fs::write(&path, src).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

/// Run a program and return its trimmed stdout, asserting it succeeded.
fn stdout_of(src: &str) -> String {
    let output = run_source(src);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .trim_end()
        .to_string()
}

#[test]
fn header_get_is_case_insensitive() {
    // The native parser lower-cases names, but a hand-built map may not, so the
    // lookup must not depend on the stored casing.
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["Content-Type"] = "text/html"
    println(header_get(h, "content-type").unwrap())
    println(header_get(h, "CONTENT-TYPE").unwrap())
    println(header_get(h, "Content-Type").unwrap())
}
"#,
    );
    assert_eq!(out, "text/html\ntext/html\ntext/html");
}

#[test]
fn header_get_returns_nil_when_absent() {
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["accept"] = "*/*"
    println(str(header_get(h, "authorization").unwrap()))
}
"#,
    );
    assert_eq!(
        out, "nil",
        "a missing optional header is normal, not an error"
    );
}

#[test]
fn bearer_token_extracts_the_credential() {
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["authorization"] = "Bearer abc.def.ghi"
    println(bearer_token(h).unwrap())
    let mixed = map()
    mixed["Authorization"] = "bearer lowercase-scheme"
    println(bearer_token(mixed).unwrap())
}
"#,
    );
    assert_eq!(out, "abc.def.ghi\nlowercase-scheme");
}

#[test]
fn bearer_token_names_each_failure() {
    let out = stdout_of(
        r#"
fn main() {
    let missing = map()
    println(bearer_token(missing).err())

    let wrong = map()
    wrong["authorization"] = "Basic dXNlcjpwYXNz"
    println(bearer_token(wrong).err())

    let bare = map()
    bare["authorization"] = "abc.def.ghi"
    println(bearer_token(bare).err())

    let empty = map()
    empty["authorization"] = "Bearer   "
    println(bearer_token(empty).err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].contains("no Authorization header"),
        "got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("Basic"),
        "should name the scheme: {}",
        lines[1]
    );
    // A bare token must be rejected, not silently accepted as a credential.
    assert!(lines[2].contains("no scheme"), "got: {}", lines[2]);
    // `Bearer` followed only by whitespace trims to a bare scheme with nothing
    // after it, so it is reported as a missing token rather than an empty one.
    // Either way it must be rejected; what matters is that it is never accepted.
    assert!(
        lines[3].contains("empty token") || lines[3].contains("no scheme"),
        "got: {}",
        lines[3]
    );
}

#[test]
fn client_ip_prefers_the_leftmost_forwarded_entry() {
    // Each proxy appends, so the original client is leftmost.
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["x-forwarded-for"] = "203.0.113.7, 70.41.3.18, 150.172.238.178"
    println(client_ip(h, "10.0.0.1"))
}
"#,
    );
    assert_eq!(out, "203.0.113.7");
}

#[test]
fn client_ip_trims_whitespace_around_entries() {
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["X-Forwarded-For"] = "   203.0.113.9   ,  70.41.3.18  "
    println(client_ip(h, "10.0.0.1"))
}
"#,
    );
    assert_eq!(out, "203.0.113.9");
}

#[test]
fn client_ip_falls_back_to_real_ip_then_remote_addr() {
    let out = stdout_of(
        r#"
fn main() {
    let real = map()
    real["x-real-ip"] = "198.51.100.4"
    println(client_ip(real, "10.0.0.1"))

    let none = map()
    println(client_ip(none, "10.0.0.1"))
}
"#,
    );
    assert_eq!(out, "198.51.100.4\n10.0.0.1");
}

#[test]
fn accepts_honors_exact_types_and_wildcards() {
    let out = stdout_of(
        r#"
fn main() {
    let exact = map()
    exact["accept"] = "application/json"
    println(str(accepts(exact, "application/json")))
    println(str(accepts(exact, "text/html")))

    let subtype = map()
    subtype["Accept"] = "text/*"
    println(str(accepts(subtype, "text/html")))
    println(str(accepts(subtype, "image/png")))

    let any = map()
    any["accept"] = "*/*"
    println(str(accepts(any, "application/octet-stream")))
}
"#,
    );
    assert_eq!(out, "true\nfalse\ntrue\nfalse\ntrue");
}

#[test]
fn accepts_handles_lists_q_values_and_a_missing_header() {
    let out = stdout_of(
        r#"
fn main() {
    let list = map()
    list["accept"] = "text/html;q=0.9, application/json;q=0.8"
    println(str(accepts(list, "application/json")))
    println(str(accepts(list, "image/png")))

    // No Accept header means the client expressed no preference.
    let none = map()
    println(str(accepts(none, "application/json")))
}
"#,
    );
    assert_eq!(out, "true\nfalse\ntrue");
}

#[test]
fn security_headers_contains_each_documented_key() {
    let out = stdout_of(
        r#"
fn main() {
    let h = security_headers()
    println(h["x-content-type-options"])
    println(h["x-frame-options"])
    println(h["referrer-policy"])
    println(h["content-security-policy"])
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "nosniff");
    assert_eq!(lines[1], "DENY");
    assert_eq!(lines[2], "strict-origin-when-cross-origin");
    assert!(lines[3].contains("default-src 'self'"), "got: {}", lines[3]);
    assert!(
        lines[3].contains("frame-ancestors 'none'"),
        "got: {}",
        lines[3]
    );
    // A policy allowing inline script would not mitigate XSS.
    assert!(
        !lines[3].contains("unsafe-inline"),
        "default CSP must not allow inline script: {}",
        lines[3]
    );
}

#[test]
fn header_helpers_reject_a_non_map_argument() {
    let output = run_source("fn main() { println(bearer_token(\"nope\")) }\n");
    assert!(!output.status.success(), "a str is not a headers map");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("headers map"),
        "error should name the expected type, got: {stderr}"
    );
}
