//! Integration coverage for the ETag and cache-header built-ins.
//!
//! The load-bearing case is `etag_matches`. A naive substring test would let
//! `"abc"` match `"abcdef"` and answer `304` for a body the client has never
//! seen, so the prefix-only near-miss is asserted explicitly alongside the
//! ordinary hits.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its trimmed stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_etag_{}", std::process::id()));
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

/// Split captured stdout into lines for per-line assertions.
fn run_lines(out: &str) -> Vec<&str> {
    out.lines().collect()
}

#[test]
fn identical_bodies_share_an_etag_and_different_bodies_differ() {
    let out = run(r#"
fn main() {
    println(str(etag_of("hello") == etag_of("hello")))
    println(str(etag_of("hello") == etag_of("hello ")))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "same body must produce the same tag");
    assert_eq!(lines[1], "false", "different bodies must differ");
}

/// An unquoted entity tag is malformed, so the quotes are part of the value.
#[test]
fn strong_etag_is_quoted_and_hex() {
    let out = run(r#"
fn main() {
    println(etag_of("hello"))
}
"#);
    assert!(out.starts_with('"'), "got: {out}");
    assert!(out.ends_with('"'), "got: {out}");
    let inner = out.trim_matches('"');
    assert_eq!(inner.len(), 64, "sha256 hex is 64 chars, got: {inner}");
    assert!(inner.chars().all(|c| c.is_ascii_hexdigit()), "got: {inner}");
}

#[test]
fn weak_etag_is_prefixed_and_wraps_the_strong_form() {
    let out = run(r#"
fn main() {
    println(etag_weak("hello"))
    println(str(etag_weak("hello") == "W/" + etag_of("hello")))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with("W/\""), "got: {}", lines[0]);
    assert_eq!(lines[1], "true");
}

#[test]
fn matches_a_single_value_and_a_list() {
    let out = run(r#"
fn main() {
    let tag = etag_of("hello")
    println(str(etag_matches(tag, tag)))
    println(str(etag_matches("\"nope\", " + tag + ", \"other\"", tag)))
    println(str(etag_matches("\"nope\", \"other\"", tag)))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "single value must match");
    assert_eq!(lines[1], "true", "list containing the tag must match");
    assert_eq!(lines[2], "false", "list without the tag must not match");
}

#[test]
fn matches_tolerates_surrounding_whitespace() {
    let out = run(r#"
fn main() {
    let tag = etag_of("hello")
    println(str(etag_matches("   " + tag + "   ", tag)))
    println(str(etag_matches("\"a\" ,  " + tag, tag)))
}
"#);
    for line in run_lines(&out) {
        assert_eq!(line, "true", "whitespace must be trimmed, got: {out}");
    }
}

/// `*` matches any current representation, per RFC 9110.
#[test]
fn wildcard_matches_any_etag() {
    let out = run(r#"
fn main() {
    println(str(etag_matches("*", etag_of("hello"))))
    println(str(etag_matches("  *  ", etag_of("anything"))))
}
"#);
    for line in run_lines(&out) {
        assert_eq!(line, "true", "got: {out}");
    }
}

/// Weak comparison: `W/"x"` and `"x"` are the same validator.
#[test]
fn weak_and_strong_forms_compare_equal() {
    let out = run(r#"
fn main() {
    let strong = etag_of("hello")
    let weak = etag_weak("hello")
    println(str(etag_matches(weak, strong)))
    println(str(etag_matches(strong, weak)))
}
"#);
    for line in run_lines(&out) {
        assert_eq!(line, "true", "got: {out}");
    }
}

/// The bug this group exists to avoid: a prefix must not count as a match.
#[test]
fn rejects_a_prefix_only_near_miss() {
    let out = run(r#"
fn main() {
    println(str(etag_matches("\"abcdef\"", "\"abc\"")))
    println(str(etag_matches("\"abc\"", "\"abcdef\"")))
    println(str(etag_matches("\"abc\", \"abcdef\"", "\"abcd\"")))
}
"#);
    for line in run_lines(&out) {
        assert_eq!(
            line, "false",
            "a prefix must never match; serving stale content is worse than not caching: {out}"
        );
    }
}

#[test]
fn empty_if_none_match_does_not_match() {
    let out = run(r#"
fn main() {
    println(str(etag_matches("", etag_of("hello"))))
    println(str(etag_matches(",  ,", etag_of("hello"))))
}
"#);
    for line in run_lines(&out) {
        assert_eq!(line, "false", "got: {out}");
    }
}

#[test]
fn cache_control_emits_directives_in_a_stable_order() {
    let out = run(r#"
fn main() {
    let a = map()
    a.public = true
    a.max_age = 31536000
    a.immutable = true
    println(cache_control(a)?)

    let b = map()
    b.private = true
    b.no_cache = true
    b.must_revalidate = true
    println(cache_control(b)?)

    let c = map()
    c.public = true
    c.max_age = 60
    c.s_maxage = 120
    println(cache_control(c)?)
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "public, max-age=31536000, immutable");
    assert_eq!(lines[1], "private, no-cache, must-revalidate");
    assert_eq!(lines[2], "public, max-age=60, s-maxage=120");
}

/// An unclassified response must not be left to heuristic caching.
#[test]
fn empty_options_default_to_no_store() {
    let out = run(r#"
fn main() {
    println(cache_control(map())?)
}
"#);
    assert_eq!(out, "no-store");
}

#[test]
fn contradictory_directives_are_named_errors() {
    let out = run(r#"
fn main() {
    let a = map()
    a.no_store = true
    a.max_age = 60
    println(str(cache_control(a).is_err()))
    println(cache_control(a).err())

    let b = map()
    b.public = true
    b.private = true
    println(cache_control(b).err())

    let c = map()
    c.max_age = "soon"
    println(cache_control(c).err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true");
    assert!(lines[1].contains("no_store"), "got: {}", lines[1]);
    assert!(lines[2].contains("mutually exclusive"), "got: {}", lines[2]);
    assert!(lines[3].contains("max_age"), "got: {}", lines[3]);
}

#[test]
fn not_modified_response_has_status_304_and_an_empty_body() {
    let out = run(r#"
fn main() {
    let resp = not_modified_response()
    println(str(resp.status))
    println("body_len=" + str(resp.body.len()))
    println("headers=" + str(resp.headers.len()))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "304");
    assert_eq!(lines[1], "body_len=0", "RFC 9110 forbids a body on 304");
    assert_eq!(lines[2], "headers=0");
}
