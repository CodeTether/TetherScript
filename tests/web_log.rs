//! Integration coverage for the structured logging built-ins.
//!
//! Two properties are load-bearing and are asserted rather than assumed:
//!
//! 1. **Lines go to stderr, never stdout.** Stdout carries HTTP response bodies
//!    and JSON-RPC frames, so a log line there corrupts the protocol. Every test
//!    reads the child's stderr and several assert stdout stayed empty.
//! 2. **Reserved keys cannot be overwritten.** A caller field named `level` must
//!    not be able to relabel its own line and hide an error.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Captured output of one script run, with the two streams kept apart.
struct Run {
    stdout: String,
    stderr: String,
}

/// Run a script, optionally with a `LOG_LEVEL`, returning both streams.
fn run_with(source: &str, log_level: Option<&str>) -> Run {
    let dir = std::env::temp_dir().join(format!("tether_log_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, source).expect("source should be writable");

    let mut command = Command::new(env!("CARGO_BIN_EXE_tetherscript"));
    command.arg("run").arg(&path);
    match log_level {
        Some(level) => {
            command.env("LOG_LEVEL", level);
        }
        // Clear it so a developer's own LOG_LEVEL cannot change the result.
        None => {
            command.env_remove("LOG_LEVEL");
        }
    }
    let output = command.output().expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn run(source: &str) -> Run {
    run_with(source, None)
}

/// The last stderr line that looks like a JSON object.
fn json_line(stderr: &str) -> &str {
    stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON line on stderr: {stderr}"))
}

#[test]
fn every_level_emits_parseable_json_with_level_and_msg() {
    for (builtin, level) in [
        ("log_debug", "debug"),
        ("log_info", "info"),
        ("log_warn", "warn"),
        ("log_error", "error"),
    ] {
        // debug must not be filtered out by the default threshold here, so the
        // level is set explicitly to the level under test.
        let out = run_with(
            &format!(
                r#"
fn main() {{
    {builtin}("hello")?
}}
"#
            ),
            Some(level),
        );
        let line = json_line(&out.stderr);
        assert!(
            line.contains(&format!("\"level\":\"{level}\"")),
            "got: {line}"
        );
        assert!(line.contains("\"msg\":\"hello\""), "got: {line}");
        assert!(line.contains("\"ts\":"), "every line needs a ts: {line}");
    }
}

#[test]
fn log_output_goes_to_stderr_and_stdout_stays_empty() {
    let out = run(r#"
fn main() {
    log_info("only on stderr")?
}
"#);
    assert!(
        out.stdout.trim().is_empty(),
        "stdout must stay clean for response bodies and JSON-RPC, got: {}",
        out.stdout
    );
    assert!(
        out.stderr.contains("only on stderr"),
        "stderr should carry the line: {}",
        out.stderr
    );
}

#[test]
fn the_returned_line_matches_what_was_emitted() {
    // The line is returned as well as emitted, so a caller can forward the same
    // bytes without re-rendering and getting a second timestamp.
    let out = run(r#"
fn main() {
    let line = log_info("returned")?
    print(line)
}
"#);
    assert_eq!(out.stdout.trim(), json_line(&out.stderr).trim());
}

#[test]
fn a_message_with_quotes_newlines_and_backslashes_round_trips() {
    // This is why the line is built with the JSON encoder instead of format!:
    // any of these characters would break a hand-built line.
    let out = run(r#"
fn main() {
    let line = log_json("info", "say \"hi\"\nnext\\path", nil)?
    let back = json_parse(line)
    println(back.msg)
}
"#);
    assert_eq!(
        out.stdout, "say \"hi\"\nnext\\path\n",
        "stderr: {}",
        out.stderr
    );
}

#[test]
fn caller_fields_appear_in_the_line() {
    let out = run(r#"
fn main() {
    let fields = map()
    fields.request_id = "abc-123"
    fields.status = 500
    let line = log_json("error", "upstream timeout", fields)?
    let back = json_parse(line)
    println(back.request_id)
    println(str(back.status))
}
"#);
    let mut lines = out.stdout.lines();
    assert_eq!(lines.next(), Some("abc-123"), "stderr: {}", out.stderr);
    assert_eq!(lines.next(), Some("500"), "stderr: {}", out.stderr);
}

#[test]
fn a_caller_field_cannot_overwrite_a_reserved_key() {
    // Shadowing `level` would let a script downgrade its own error line.
    let out = run(r#"
fn main() {
    let fields = map()
    fields.level = "debug"
    fields.msg = "spoofed"
    let line = log_json("error", "real message", fields)?
    let back = json_parse(line)
    println(back.level)
    println(back.msg)
}
"#);
    let mut lines = out.stdout.lines();
    assert_eq!(lines.next(), Some("error"), "stderr: {}", out.stderr);
    assert_eq!(lines.next(), Some("real message"), "stderr: {}", out.stderr);
}

#[test]
fn an_unknown_level_is_a_named_error() {
    let out = run(r#"
fn main() {
    let bad = log_json("warning", "typo", nil)
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let mut lines = out.stdout.lines();
    assert_eq!(lines.next(), Some("true"), "stderr: {}", out.stderr);
    let message = lines.next().unwrap_or_default();
    assert!(
        message.contains("warning"),
        "should name the level: {message}"
    );
    assert!(message.contains("unknown level"), "got: {message}");
}

#[test]
fn non_map_fields_are_rejected_by_name() {
    let out = run(r#"
fn main() {
    let bad = log_json("info", "msg", "not-a-map")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let mut lines = out.stdout.lines();
    assert_eq!(lines.next(), Some("true"), "stderr: {}", out.stderr);
    let message = lines.next().unwrap_or_default();
    assert!(message.contains("map"), "got: {message}");
}

#[test]
fn log_level_enabled_filters_at_the_default_threshold() {
    // Documented default is info, so debug is off and warn is on.
    let out = run(r#"
fn main() {
    println(str(log_level_enabled("debug")?))
    println(str(log_level_enabled("info")?))
    println(str(log_level_enabled("warn")?))
}
"#);
    assert_eq!(
        out.stdout.lines().collect::<Vec<_>>(),
        vec!["false", "true", "true"],
        "stderr: {}",
        out.stderr
    );
}

#[test]
fn log_level_enabled_honours_the_environment_variable() {
    let source = r#"
fn main() {
    println(str(log_level_enabled("debug")?))
    println(str(log_level_enabled("error")?))
}
"#;
    let debug = run_with(source, Some("debug"));
    assert_eq!(
        debug.stdout.lines().collect::<Vec<_>>(),
        vec!["true", "true"],
        "LOG_LEVEL=debug should admit debug"
    );

    let quiet = run_with(source, Some("error"));
    assert_eq!(
        quiet.stdout.lines().collect::<Vec<_>>(),
        vec!["false", "true"],
        "LOG_LEVEL=error should drop debug"
    );
}

#[test]
fn an_unparseable_log_level_falls_back_to_the_default() {
    // A typo in deployment config must not silence every line.
    let out = run_with(
        r#"
fn main() {
    println(str(log_level_enabled("info")?))
    println(str(log_level_enabled("debug")?))
}
"#,
        Some("not-a-level"),
    );
    assert_eq!(
        out.stdout.lines().collect::<Vec<_>>(),
        vec!["true", "false"],
        "stderr: {}",
        out.stderr
    );
}

#[test]
fn level_names_are_case_insensitive() {
    let out = run(r#"
fn main() {
    let line = log_json("ERROR", "shouted", nil)?
    let back = json_parse(line)
    println(back.level)
}
"#);
    assert_eq!(out.stdout.trim(), "error", "stderr: {}", out.stderr);
}
