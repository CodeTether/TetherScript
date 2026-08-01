//! Integration coverage for the token-bucket rate limiter.
//!
//! Two properties carry the most weight. Tokens must never exceed capacity, or an
//! idle client banks an unbounded burst and the limit stops meaning anything. And
//! `retry_after_header` must round *up*, because rounding down tells a client to
//! retry while it is still limited — a retry storm exactly when the server is
//! already saturated.
//!
//! Every case runs a real `.tether` script through the binary, so these exercise
//! the registered built-ins rather than the Rust functions behind them.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its trimmed stdout, asserting it succeeded.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_ratelimit_{}", std::process::id()));
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
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn a_fresh_bucket_allows_exactly_capacity_then_denies() {
    // Refill is 1/sec, so within a few milliseconds no meaningful refill occurs
    // and the third take must be denied.
    let out = run(r#"fn main() {
    let mut b = bucket_new(2, 1)?
    let mut taken = 0
    let mut i = 0
    while i < 3 {
        let t = bucket_take(b, 1)?
        b = t.bucket
        if t.allowed { taken = taken + 1 }
        i = i + 1
    }
    println(str(taken))
}"#);
    assert_eq!(out, "2", "a capacity-2 bucket must admit exactly 2");
}

#[test]
fn a_denied_take_reports_a_positive_retry_after() {
    let out = run(r#"fn main() {
    let mut b = bucket_new(1, 1)?
    let first = bucket_take(b, 1)?
    b = first.bucket
    let second = bucket_take(b, 1)?
    println(str(second.allowed))
    println(str(second.retry_after_ms > 0))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "second take must be denied: {out}");
    assert_eq!(lines[1], "true", "retry_after_ms must be positive: {out}");
}

#[test]
fn tokens_refill_after_elapsed_time() {
    // Drain the bucket, wait past one refill period, then take again. A limiter
    // that never refilled would stay denied forever.
    let out = run(r#"fn main() {
    let mut b = bucket_new(1, 20)?
    let first = bucket_take(b, 1)?
    b = first.bucket
    let denied = bucket_take(b, 1)?
    b = denied.bucket
    sleep_ms(200)
    let after = bucket_take(b, 1)?
    println(str(denied.allowed))
    println(str(after.allowed))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "immediate retry must be denied: {out}");
    assert_eq!(lines[1], "true", "must refill after waiting: {out}");
}

#[test]
fn tokens_never_exceed_capacity_after_a_long_idle() {
    // At 100/sec a 200ms idle would earn 20 tokens, but capacity is 2, so an
    // unclamped bucket would admit far more than 2 requests here.
    let out = run(r#"fn main() {
    let mut b = bucket_new(2, 100)?
    sleep_ms(200)
    let mut taken = 0
    let mut i = 0
    while i < 6 {
        let t = bucket_take(b, 1)?
        b = t.bucket
        if t.allowed { taken = taken + 1 }
        i = i + 1
    }
    println(str(taken <= 3))
    println(str(taken >= 2))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0], "true",
        "an idle bucket must stay clamped to capacity: {out}"
    );
    assert_eq!(lines[1], "true", "capacity must still be usable: {out}");
}

#[test]
fn a_cost_larger_than_capacity_is_a_named_error() {
    let out = run(r#"fn main() {
    let b = bucket_new(5, 1)?
    let bad = bucket_take(b, 6)
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "cost above capacity must error: {out}");
    assert!(
        lines[1].contains("exceeds capacity"),
        "error must explain why it can never succeed, got: {}",
        lines[1]
    );
}

#[test]
fn a_nonpositive_capacity_or_refill_is_a_named_error() {
    let out = run(r#"fn main() {
    let zero = bucket_new(0, 1)
    let negative = bucket_new(-1, 1)
    let no_refill = bucket_new(5, 0)
    println(zero.err())
    println(str(negative.is_err()))
    println(no_refill.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].contains("capacity"),
        "must name capacity, got: {}",
        lines[0]
    );
    assert_eq!(lines[1], "true", "negative capacity must error: {out}");
    assert!(
        lines[2].contains("refill_per_sec"),
        "must name refill_per_sec, got: {}",
        lines[2]
    );
}

#[test]
fn retry_after_header_rounds_up_to_whole_seconds() {
    let out = run(r#"fn main() {
    println(str(retry_after_header(1)))
    println(str(retry_after_header(1001)))
    println(str(retry_after_header(1000)))
    println(str(retry_after_header(0)))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "1", "1ms must round up to 1s: {out}");
    assert_eq!(lines[1], "2", "1001ms must round up to 2s: {out}");
    assert_eq!(lines[2], "1", "an exact second must stay 1s: {out}");
    assert_eq!(lines[3], "0", "no wait must be 0s: {out}");
}

#[test]
fn too_many_requests_response_is_429_with_a_retry_after_header() {
    let out = run(r#"fn main() {
    let resp = too_many_requests_response(1500)
    println(str(resp.status))
    println(str(resp.headers["retry-after"]))
    println(str(resp.body.len() > 0))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "429", "status must be 429: {out}");
    assert_eq!(lines[1], "2", "1500ms must advertise 2s: {out}");
    assert_eq!(lines[2], "true", "body should explain the refusal: {out}");
}

#[test]
fn the_returned_bucket_must_be_persisted_or_the_limit_never_applies() {
    // Discarding the returned bucket is the likeliest caller mistake, so this
    // pins the difference: reusing the original always sees a full bucket.
    let out = run(r#"fn main() {
    let b = bucket_new(1, 1)?
    let first = bucket_take(b, 1)?
    let ignored = bucket_take(b, 1)?
    println(str(first.allowed))
    println(str(ignored.allowed))
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "first take succeeds: {out}");
    assert_eq!(
        lines[1], "true",
        "reusing the stale bucket sees it still full, which is why the caller must persist: {out}"
    );
}

#[test]
fn a_malformed_bucket_names_the_missing_field() {
    let out = run(r#"fn main() {
    let fake = map()
    fake.capacity = 5
    let bad = bucket_take(fake, 1)
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "an incomplete bucket must error: {out}");
    assert!(
        lines[1].contains("refill_per_sec") || lines[1].contains("missing"),
        "error must name the missing field, got: {}",
        lines[1]
    );
}

#[test]
fn a_non_map_bucket_is_rejected_by_type() {
    let out = run(r#"fn main() {
    let bad = bucket_take("not a bucket", 1)
    println(bad.err())
}"#);
    assert!(
        out.contains("must be a map") && out.contains("str"),
        "error must name the expected and actual type, got: {out}"
    );
}
