//! Integration coverage for the server-sent events framing built-ins.
//!
//! The load-bearing detail is multi-line `data`. A raw newline terminates an SSE
//! field, so a two-line payload emitted as a single `data:` line is parsed as a
//! truncated event. Every assertion below pins the exact bytes on the wire,
//! because "looks about right" framing still breaks real clients.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its stdout, trimmed of the trailing newline only.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_sse_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, source).expect("source should be writable");
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Render a frame with escaped newlines so exact bytes are visible in failures.
fn escaped(source: &str) -> String {
    run(source).trim_end_matches('\n').to_string()
}

#[test]
fn single_line_data_frame_is_exact() {
    // `escape` is done in-script: print the frame with newlines shown as \n.
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "hello"
    let frame = sse_event(f).unwrap()
    println(frame.replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out, "data: hello\\n\\n", "full output: {out}");
}

#[test]
fn multi_line_data_emits_one_data_line_per_line() {
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "first\nsecond\nthird"
    let frame = sse_event(f).unwrap()
    println(frame.replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "data: first\\ndata: second\\ndata: third\\n\\n",
        "each line needs its own data: prefix, else the event truncates: {out}"
    );
}

#[test]
fn event_id_and_retry_appear_in_stable_order() {
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "payload"
    f.retry = 3000
    f.id = "42"
    f.event = "tick"
    let frame = sse_event(f).unwrap()
    println(frame.replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "event: tick\\nid: 42\\nretry: 3000\\ndata: payload\\n\\n",
        "order must be event, id, retry, data regardless of insertion order: {out}"
    );
}

#[test]
fn frame_terminates_with_a_blank_line() {
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "x"
    let frame = sse_event(f).unwrap()
    println(str(frame.ends_with("\n\n")))
}
"#,
    );
    assert_eq!(out, "true", "the blank line is what dispatches the event");
}

#[test]
fn crlf_in_data_leaves_no_stray_carriage_return() {
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "one\r\ntwo"
    let frame = sse_event(f).unwrap()
    println(str(frame.contains("\r")))
    println(frame.replace("\n", "\\n"))
}
"#,
    );
    let mut lines = out.lines();
    assert_eq!(
        lines.next(),
        Some("false"),
        "a CR inside a frame corrupts the field: {out}"
    );
    assert_eq!(
        lines.next(),
        Some("data: one\\ndata: two\\n\\n"),
        "CRLF must split into two data lines: {out}"
    );
}

#[test]
fn non_integer_retry_is_a_named_error() {
    let out = escaped(
        r#"
fn main() {
    let f = map()
    f.data = "x"
    f.retry = "soon"
    let r = sse_event(f)
    println(str(r.is_err()))
    println(r.err())
}
"#,
    );
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("true"), "full output: {out}");
    let message = lines.next().unwrap_or_default();
    assert!(
        message.contains("retry"),
        "error must name the offending field: {message}"
    );
}

#[test]
fn comment_is_a_single_prefixed_line() {
    let out = escaped(
        r#"
fn main() {
    println(sse_comment("keep-alive").unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out, ": keep-alive\\n", "full output: {out}");
}

#[test]
fn retry_builds_a_bare_frame() {
    let out = escaped(
        r#"
fn main() {
    println(sse_retry(5000).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out, "retry: 5000\\n\\n", "full output: {out}");
}
