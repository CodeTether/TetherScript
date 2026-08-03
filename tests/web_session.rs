//! Behaviour tests for the signed session-cookie built-ins.
//!
//! Every value here is produced by `session_sign` and then manipulated in-script,
//! so no expected signature is invented. Forged values are built by re-encoding a
//! real payload or tag, which is how an attacker would produce them.
//!
//! These drive the built-ins through the interpreter, because that is the surface
//! a real session port actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_session_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("session_case_{case}.tether"));
    std::fs::write(&path, src).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

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
fn round_trip_preserves_several_keys() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "abc123"
    payload.user_id = "user-1"
    payload.role = "admin"
    payload.count = 3

    let value = session_sign(payload, "secret").unwrap()
    let back = session_verify(value, "secret").unwrap()
    println(back.sid)
    println(back.user_id)
    println(back.role)
    println(str(back.count))
}
"#,
    );
    assert_eq!(out, "abc123\nuser-1\nadmin\n3", "full output: {out}");
}

#[test]
fn wrong_secret_is_a_named_error() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "abc123"
    let value = session_sign(payload, "right-secret").unwrap()
    let bad = session_verify(value, "wrong-secret")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("signature does not match"),
        "error should name the signature, got: {}",
        lines[1]
    );
}

/// Re-encoding a different payload against the original tag must fail.
#[test]
fn tampered_payload_is_a_named_error() {
    let out = stdout_of(
        r#"
fn main() {
    let mine = map()
    mine.role = "user"
    let value = session_sign(mine, "secret").unwrap()
    let parts = value.split(".")

    let forged = map()
    forged.role = "admin"
    let elevated = session_sign(forged, "attacker-key").unwrap()
    let swapped = elevated.split(".")[0] + "." + parts[1]

    let bad = session_verify(swapped, "secret")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("signature does not match"),
        "error should name the signature, got: {}",
        lines[1]
    );
}

#[test]
fn tampered_tag_is_a_named_error() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "abc123"
    let value = session_sign(payload, "secret").unwrap()
    let parts = value.split(".")

    // A valid-looking but wrong tag: the payload segment re-signed with another key.
    let other = session_sign(payload, "other-secret").unwrap()
    let forged = parts[0] + "." + other.split(".")[1]

    let bad = session_verify(forged, "secret")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("signature does not match"),
        "error should name the signature, got: {}",
        lines[1]
    );
}

#[test]
fn value_carries_no_base64_padding() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "a"
    payload.user_id = "bb"
    let value = session_sign(payload, "secret").unwrap()
    println(str(value.contains("=")))
    println(str(value.split(".").len()))
}
"#,
    );
    assert_eq!(out, "false\n2", "value must be unpadded, got: {out}");
}

#[test]
fn touch_moves_exp_forward_and_leaves_other_keys() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "abc123"
    payload.role = "admin"
    payload.exp = 1

    let refreshed = session_touch(payload, 604800).unwrap()
    println(refreshed.sid)
    println(refreshed.role)
    // Use the core clock, not another group's time_now_secs, so this test does
    // not depend on whether the datetime group is registered.
    println(str(refreshed.exp > time_now_ms() / 1000))
    println(str(refreshed.exp > 1))
}
"#,
    );
    assert_eq!(
        out, "abc123\nadmin\ntrue\ntrue",
        "touch must refresh exp only, got: {out}"
    );
}

#[test]
fn expired_is_true_for_a_past_exp_and_false_for_a_future_one() {
    let out = stdout_of(
        r#"
fn main() {
    let past = map()
    past.exp = 1
    println(str(session_expired(past).unwrap()))

    let future = map()
    future.exp = time_now_ms() / 1000 + 3600
    println(str(session_expired(future).unwrap()))

    // No exp at all means the payload carries no lifetime of its own.
    let none = map()
    none.sid = "x"
    println(str(session_expired(none).unwrap()))
}
"#,
    );
    assert_eq!(out, "true\nfalse\nfalse", "full output: {out}");
}

/// The signed value must survive the real cookie round trip it is built for.
#[test]
fn signed_value_survives_cookie_serialize_and_parse() {
    let out = stdout_of(
        r#"
fn main() {
    let payload = map()
    payload.sid = "abc123"
    payload.user_id = "user-1"
    let value = session_sign(payload, "secret").unwrap()

    // The reference config: HttpOnly, SameSite=Lax, Path=/, 7-day TTL.
    let opts = map()
    opts.path = "/"
    opts.http_only = true
    opts.same_site = "Lax"
    opts.max_age = 604800
    let header = cookie_serialize("id", value, opts).unwrap()
    println(str(header.contains("HttpOnly")))
    println(str(header.contains("SameSite=Lax")))

    // Feed the Set-Cookie name=value pair back through the request-side parser.
    let jar = cookie_parse(header.split(";")[0])
    let back = session_verify(jar["id"], "secret").unwrap()
    println(back.sid)
    println(back.user_id)
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\nabc123\nuser-1",
        "signed value must round trip through a real cookie, got: {out}"
    );
}

#[test]
fn malformed_values_are_named_errors() {
    let out = stdout_of(
        r#"
fn main() {
    println(session_verify("no-separator", "secret").err())
    println(session_verify("a.b.c", "secret").err())
    println(session_verify("not+base64url.tag", "secret").err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("segments"), "got: {}", lines[0]);
    assert!(lines[1].contains("segments"), "got: {}", lines[1]);
    // The tag is checked before the payload is decoded, so a value with an
    // undecodable payload is reported as a signature failure. That ordering is
    // deliberate: an unauthenticated payload should never be parsed at all.
    assert!(
        lines[2].contains("base64url") || lines[2].contains("signature"),
        "should name the encoding or the signature, got: {}",
        lines[2]
    );
}

#[test]
fn wrong_argument_types_name_the_parameter() {
    let out = stdout_of(
        r#"
fn main() {
    println(session_sign("not-a-map", "secret").err())
    println(session_touch(map(), "not-an-int").err())
    println(session_expired("not-a-map").err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].contains("payload must be map"),
        "got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("ttl_seconds must be int"),
        "got: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("payload must be map"),
        "got: {}",
        lines[2]
    );
}
