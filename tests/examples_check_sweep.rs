//! Every shipped example must survive lex, parse, and ownership analysis.
//!
//! Several examples embedded JS/CSS/JSON payloads containing raw `{`, which
//! tetherscript reads as the start of a string-interpolation hole. Those files
//! failed to lex while the test suite stayed green, because nothing asserted that
//! examples are well-formed. This sweep closes that hole.

use std::path::Path;
use std::process::Command;

/// `examples/use_after_move.tether` documents a rejected program: the ownership
/// pass is supposed to reject it, so a failure there is the correct outcome.
const EXPECTED_TO_FAIL: &[&str] = &["use_after_move.tether"];

fn check(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("check")
        .arg(path)
        .output()
        .unwrap_or_else(|err| panic!("failed to check {}: {err}", path.display()))
}

fn sweep(dir: &str) {
    let mut failures = Vec::new();
    let mut checked = 0usize;

    let entries = std::fs::read_dir(dir).unwrap_or_else(|err| panic!("{dir} should exist: {err}"));
    for entry in entries {
        let path = entry.expect("directory entry should be readable").path();
        if path.extension().and_then(|e| e.to_str()) != Some("tether") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("example name should be UTF-8")
            .to_string();

        let output = check(&path);
        let expected_failure = EXPECTED_TO_FAIL.contains(&name.as_str());
        checked += 1;

        if output.status.success() == expected_failure {
            failures.push(format!(
                "{}: expected {}, got {}\n  {}",
                path.display(),
                if expected_failure { "rejection" } else { "ok" },
                if output.status.success() {
                    "ok"
                } else {
                    "rejection"
                },
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }

    assert!(checked > 0, "{dir} should contain .tether examples");
    assert!(
        failures.is_empty(),
        "{} of {checked} examples in {dir} did not check as expected:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_examples_pass_check() {
    sweep("examples");
}

#[test]
fn all_experiment_examples_pass_check() {
    sweep("experiments/examples");
}
