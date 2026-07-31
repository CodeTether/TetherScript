//! Regression coverage for string escapes that broke two shipped examples.
//!
//! `examples/json.tether` used single-quoted strings (never valid in tetherscript)
//! and `examples/tetherscript_extension.tether` embedded a raw `{`, which opens a
//! string-interpolation hole. These tests pin the escapes that make both work.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_escape_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("escape_case_{case}.tether"));
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
fn escaped_quotes_survive_in_string_literals() {
    let out = stdout_of("fn main() { println(\"say \\\"hi\\\" now\") }\n");
    assert_eq!(out, "say \"hi\" now");
}

#[test]
fn escaped_braces_are_literal_not_interpolation() {
    let out = stdout_of("fn main() { println(\"fn main() \\{ ok \\}\") }\n");
    assert_eq!(out, "fn main() { ok }");
}

#[test]
fn unescaped_brace_still_interpolates() {
    let out = stdout_of("fn main() { println(\"sum {1 + 1}\") }\n");
    assert_eq!(out, "sum 2");
}

#[test]
fn json_payload_with_escaped_braces_and_quotes_parses() {
    let src = "fn main() {\n    \
        let d = json_parse(\"\\{\\\"name\\\":\\\"TetherScript\\\"\\}\")\n    \
        println(d.name)\n}\n";
    assert_eq!(stdout_of(src), "TetherScript");
}

#[test]
fn single_quoted_strings_are_rejected_with_a_named_error() {
    let output = run_source("fn main() { println('nope') }\n");
    assert!(
        !output.status.success(),
        "single-quoted strings must not lex"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected character"),
        "error should name the offending character, got: {stderr}"
    );
}
