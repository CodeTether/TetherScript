//! Integration coverage for the streaming server-sent-events built-ins.
//!
//! Every assertion pins exact bytes. SSE has no error channel: a malformed frame
//! does not fail, it just delivers wrong or truncated data over a connection that
//! looks perfectly healthy. The two cases that matter most are multi-line `data`
//! (emitted as one line, the client silently truncates at the first newline) and
//! `cache-control: no-store` (without it, a cached stream is served stale
//! forever).
//!
//! Frames are printed with `\n` escaped to `\\n` so a failure message shows the
//! real wire bytes rather than a multi-line blob.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_ssestream_{}", std::process::id()));
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

/// Run and trim the single trailing newline `println` adds.
fn one(source: &str) -> String {
    run(source).trim_end_matches('\n').to_string()
}

/// Collect stdout lines, for multi-assertion scripts.
fn rows(source: &str) -> Vec<String> {
    run(source).lines().map(str::to_string).collect()
}

#[test]
fn single_field_event_has_exact_bytes() {
    let out = one(
        r#"
fn main() {
    let e = map()
    e.data = "hello"
    println(sse_chunk(e).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "data: hello\\n\\n",
        "one data line plus the dispatching blank line: {out}"
    );
}

#[test]
fn multi_line_data_emits_one_data_line_per_line() {
    let out = one(
        r#"
fn main() {
    let e = map()
    e.data = "alpha\nbeta\ngamma"
    println(sse_chunk(e).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "data: alpha\\ndata: beta\\ndata: gamma\\n\\n",
        "a single data: line would make the client truncate at `alpha`: {out}"
    );
}

#[test]
fn crlf_and_lone_cr_normalize_to_lf_delimited_data_lines() {
    let out = rows(
        r#"
fn main() {
    let crlf = map()
    crlf.data = "one\r\ntwo"
    let a = sse_chunk(crlf).unwrap()
    println(str(a.contains("\r")))
    println(a.replace("\n", "\\n"))

    let cr = map()
    cr.data = "one\rtwo"
    let b = sse_chunk(cr).unwrap()
    println(str(b.contains("\r")))
    println(b.replace("\n", "\\n"))

    println(str(a == b))
}
"#,
    );
    assert_eq!(out[0], "false", "a CR inside a frame corrupts the field");
    assert_eq!(out[1], "data: one\\ndata: two\\n\\n", "CRLF must split");
    assert_eq!(out[2], "false", "a lone CR must not survive");
    assert_eq!(out[3], "data: one\\ndata: two\\n\\n", "lone CR must split");
    assert_eq!(
        out[4], "true",
        "all three newline conventions must produce identical wire bytes"
    );
}

#[test]
fn id_event_and_data_appear_in_fixed_field_order() {
    let out = one(
        r#"
fn main() {
    let e = map()
    e.data = "payload"
    e.id = "17"
    e.event = "tick"
    println(sse_chunk(e).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "event: tick\\nid: 17\\ndata: payload\\n\\n",
        "order is event, id, data regardless of insertion order: {out}"
    );
}

#[test]
fn retry_field_is_placed_before_data() {
    let out = one(
        r#"
fn main() {
    let e = map()
    e.data = "p"
    e.retry = 2500
    e.id = "1"
    e.event = "n"
    println(sse_chunk(e).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, "event: n\\nid: 1\\nretry: 2500\\ndata: p\\n\\n",
        "data is last because it is the only multi-line field: {out}"
    );
}

/// The client ignores an `id` containing NUL, silently replaying from a stale
/// position after reconnect, so it must be rejected rather than emitted.
#[test]
fn id_containing_nul_is_rejected_by_name() {
    let out = rows(
        r#"
fn main() {
    let nul = hex_decode("00").unwrap()
    let e = map()
    e.data = "x"
    e.id = "4" + nul + "2"
    let r = sse_chunk(e)
    println(str(r.is_err()))
    println(r.err())
}
"#,
    );
    assert_eq!(out[0], "true", "a NUL in id must be an error");
    assert!(
        out[1].contains("NUL") && out[1].contains("id"),
        "the error must name the field and the cause: {}",
        out[1]
    );
}

#[test]
fn newline_in_event_name_is_rejected_as_field_injection() {
    let out = rows(
        r#"
fn main() {
    let e = map()
    e.data = "x"
    e.event = "a\ndata: forged"
    let r = sse_chunk(e)
    println(str(r.is_err()))
    println(r.err())
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("event"),
        "the error must name the offending field: {}",
        out[1]
    );
}

#[test]
fn keepalive_is_a_comment_only_frame() {
    let out = one(
        r#"
fn main() {
    println(sse_keepalive().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(
        out, ": keepalive\\n\\n",
        "a comment starts with ':' and dispatches nothing: {out}"
    );
}

/// Without periodic keepalives an idle stream is indistinguishable from a dead
/// one, and a buffering proxy will hold or drop it. The frame must therefore
/// carry no dispatchable field at all.
#[test]
fn keepalive_carries_no_dispatchable_field() {
    let out = rows(
        r#"
fn main() {
    let frame = sse_keepalive()
    println(str(frame.starts_with(":")))
    println(str(frame.contains("data:")))
    println(str(frame.contains("event:")))
}
"#,
    );
    assert_eq!(out[0], "true", "a keepalive must be a comment line");
    assert_eq!(out[1], "false", "a keepalive must dispatch nothing");
    assert_eq!(out[2], "false", "a keepalive must dispatch nothing");
}

#[test]
fn retry_frame_is_a_bare_directive() {
    let out = one(
        r#"
fn main() {
    println(sse_retry_frame(4000).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out, "retry: 4000\\n\\n", "full output: {out}");
}

#[test]
fn retry_frame_rejects_a_non_integer_and_a_negative_delay() {
    let out = rows(
        r#"
fn main() {
    let a = sse_retry_frame("soon")
    println(str(a.is_err()))
    println(a.err())
    let b = sse_retry_frame(-1)
    println(str(b.is_err()))
    println(b.err())
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(out[1].contains("retry"), "got: {}", out[1]);
    assert_eq!(
        out[2], "true",
        "a negative delay is silently ignored by the client"
    );
    assert!(out[3].contains("retry"), "got: {}", out[3]);
}

#[test]
fn stream_headers_declare_event_stream_no_store_and_keep_alive() {
    let out = rows(
        r#"
fn main() {
    let h = sse_stream_headers()
    println(h["content-type"])
    println(h["cache-control"])
    println(h["connection"])
    println(h["x-accel-buffering"])
}
"#,
    );
    assert_eq!(out[0], "text/event-stream; charset=utf-8");
    assert_eq!(
        out[1], "no-store",
        "a cached event stream is served stale forever"
    );
    assert_eq!(out[2], "keep-alive");
    assert_eq!(
        out[3], "no",
        "a buffering proxy turns a live stream into a batch"
    );
}

#[test]
fn batch_response_carries_status_headers_and_framed_body() {
    let out = rows(
        r#"
fn main() {
    let a = map()
    a.event = "tick"
    a.data = "1"
    let b = map()
    b.data = "two\nlines"
    let resp = sse_stream_response([a, b]).unwrap()
    println(str(resp.status))
    println(resp.headers["content-type"])
    println(resp.headers["cache-control"])
    println(resp.body.replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out[0], "200");
    assert_eq!(out[1], "text/event-stream; charset=utf-8");
    assert_eq!(out[2], "no-store", "no-store is mandatory on a stream");
    assert_eq!(
        out[3], "event: tick\\ndata: 1\\n\\ndata: two\\ndata: lines\\n\\n",
        "events concatenate in list order, each keeping its terminator: {}",
        out[3]
    );
}

/// A zero-event stream is a legitimate response — a feed with nothing to say yet
/// — so it must produce a valid empty body, not an error.
#[test]
fn empty_event_list_produces_a_valid_empty_stream() {
    let out = rows(
        r#"
fn main() {
    let resp = sse_stream_response([]).unwrap()
    println(str(resp.status))
    println("body_len=" + str(resp.body.len()))
    println(resp.headers["cache-control"])
}
"#,
    );
    assert_eq!(out[0], "200", "an empty stream is still a 200");
    assert_eq!(
        out[1], "body_len=0",
        "no events means no bytes, not an error"
    );
    assert_eq!(out[2], "no-store");
}

#[test]
fn batch_response_names_the_index_of_a_malformed_event() {
    let out = rows(
        r#"
fn main() {
    let ok = map()
    ok.data = "fine"
    let bad = map()
    bad.retry = "soon"
    let r = sse_stream_response([ok, bad])
    println(str(r.is_err()))
    println(r.err())
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains('1') && out[1].contains("retry"),
        "a bad element in a long list must be locatable: {}",
        out[1]
    );
}

#[test]
fn non_list_events_argument_is_a_named_error() {
    let out = rows(
        r#"
fn main() {
    let r = sse_stream_response("not a list")
    println(str(r.is_err()))
    println(r.err())
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("list"),
        "the error must say what was expected: {}",
        out[1]
    );
}

#[test]
fn every_chunk_ends_with_the_dispatching_blank_line() {
    let out = rows(
        r#"
fn main() {
    let e = map()
    e.data = "x"
    println(str(sse_chunk(e).unwrap().ends_with("\n\n")))
    println(str(sse_keepalive().ends_with("\n\n")))
    println(str(sse_retry_frame(1000).unwrap().ends_with("\n\n")))
}
"#,
    );
    for row in &out {
        assert_eq!(
            row.as_str(),
            "true",
            "a frame without its blank line is an event that never fires"
        );
    }
}

/// An empty payload is a valid event carrying the empty string, so it still needs
/// exactly one `data:` line.
#[test]
fn empty_data_payload_still_emits_one_data_line() {
    let out = one(
        r#"
fn main() {
    let e = map()
    e.data = ""
    println(sse_chunk(e).unwrap().replace("\n", "\\n"))
}
"#,
    );
    assert_eq!(out, "data: \\n\\n", "full output: {out}");
}
