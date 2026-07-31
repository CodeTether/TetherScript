use std::process::Command;

/// The self-extension plugin example embedded a raw `{` inside a string, which
/// opens an interpolation hole and failed to lex. It now escapes the braces, so
/// the example must at least pass lex, parse, and ownership analysis.
#[test]
fn tetherscript_extension_example_passes_check() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["check", "examples/tetherscript_extension.tether"])
        .output()
        .expect("check should run");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
