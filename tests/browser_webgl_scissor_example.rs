use std::process::Command;

#[test]
fn browser_webgl_scissor_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_webgl_scissor.tether"])
        .output()
        .expect("browser WebGL scissor example should run");

    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/browser_webgl_scissor.stdout")
    );
}
