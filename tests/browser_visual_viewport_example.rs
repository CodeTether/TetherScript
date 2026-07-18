use std::process::Command;

#[test]
fn browser_visual_viewport_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_visual_viewport.tether"])
        .output()
        .expect("browser visual viewport example should run");

    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/browser_visual_viewport.stdout")
    );
}
