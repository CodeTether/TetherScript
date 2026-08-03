//! Coverage for the Actix-compatible route matching built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: the matcher is a private submodule, and the script
//! surface is what a real dispatcher actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_route_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("route_case_{case}.tether"));
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
fn literal_pattern_matches_and_captures_nothing() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/healthz", "/healthz")?
    println(str(m != nil))
    println(str(m.len()))
}
"#,
    );
    assert_eq!(out, "true\n0");
}

/// A miss is an ordinary outcome, so it is nil rather than an Err.
#[test]
fn literal_pattern_that_does_not_match_is_nil_not_error() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/healthz", "/readyz")?
    println(str(m == nil))
}
"#,
    );
    assert_eq!(out, "true");
}

#[test]
fn captures_a_single_parameter() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/customers/\{id\}", "/customers/42")?
    println(m.id)
}
"#,
    );
    assert_eq!(out, "42");
}

#[test]
fn captures_multiple_parameters() {
    let out = stdout_of(
        r#"
fn main() {
    let p = "/ab-tests/\{id\}/variants/\{variant_id\}"
    let m = route_match(p, "/ab-tests/7/variants/v9")?
    println(m.id + "," + m.variant_id)
}
"#,
    );
    assert_eq!(out, "7,v9");
}

/// The property that makes `{name}` safe: it must never span a separator.
#[test]
fn a_parameter_does_not_swallow_a_slash() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/customers/\{id\}", "/customers/42/orders")?
    println(str(m == nil))
}
"#,
    );
    assert_eq!(out, "true");
}

#[test]
fn tail_capture_takes_the_remainder_including_slashes() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/static/\{rest:.*\}", "/static/css/site.css")?
    println(m.rest)
}
"#,
    );
    assert_eq!(out, "css/site.css");
}

#[test]
fn segment_count_mismatch_does_not_match() {
    let out = stdout_of(
        r#"
fn main() {
println(str(route_match("/a/\{b\}", "/a")? == nil))
    println(str(route_match("/a", "/a/b")? == nil))
}
"#,
    );
    assert_eq!(out, "true\ntrue");
}

/// `/customers/a%20b` must capture `a b`.
#[test]
fn captures_are_percent_decoded() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/customers/\{id\}", "/customers/a%20b")?
    println(m.id)
}
"#,
    );
    assert_eq!(out, "a b");
}

/// Documented choice: a trailing slash is not significant.
#[test]
fn trailing_slash_is_not_significant() {
    let out = stdout_of(
        r#"
fn main() {
    let m = route_match("/customers/\{id\}", "/customers/7/")?
    println(m.id)
    println(str(route_match("/healthz", "/healthz/")? != nil))
}
"#,
    );
    assert_eq!(out, "7\ntrue");
}

#[test]
fn empty_and_root_paths_behave() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(route_match("/", "/")? != nil))
    println(str(route_match("/", "")? != nil))
    println(str(path_segments("")?.len()))
}
"#,
    );
    assert_eq!(out, "true\ntrue\n0");
}

#[test]
fn route_params_lists_names_in_order() {
    let out = stdout_of(
        r#"
fn main() {
    let names = route_params("/ab-tests/\{id\}/variants/\{variant_id\}")?
    println(names.join(","))
    println(str(route_params("/healthz")?.len()))
    println(route_params("/static/\{rest:.*\}")?.join(","))
}
"#,
    );
    assert_eq!(out, "id,variant_id\n0\nrest");
}

#[test]
fn path_segments_drops_empties_and_decodes() {
    let out = stdout_of(
        r#"
fn main() {
    println(path_segments("/customers/a%20b")?.join("|"))
    println(path_segments("//a//b/")?.join("|"))
}
"#,
    );
    assert_eq!(out, "customers|a b\na|b");
}

/// A malformed *pattern* is a program bug, so it must be an Err, not a miss.
#[test]
fn malformed_pattern_is_an_error_naming_the_problem() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = route_match("/a/\{\}", "/a/b")
    println(str(bad.is_err()))
    let mid = route_match("/a/\{rest:.*\}/b", "/a/x/b")
    println(str(mid.is_err()))
    let re = route_match("/a/\{id:[0-9]+\}", "/a/1")
    println(str(re.is_err()))
}
"#,
    );
    assert_eq!(out, "true\ntrue\ntrue");
}
