use std::process::Command;

/// The JSON example previously used single-quoted strings, which tetherscript does
/// not accept, so it failed to lex. It now uses double quotes with `\{`/`\}`
/// escapes for the literal braces in the embedded JSON payloads.
#[test]
fn json_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/json.tether"])
        .output()
        .expect("json example should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        include_str!("examples/json.stdout")
    );
}
