use std::process::Command;

#[test]
fn browser_screen_orientation_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_screen_orientation.tether"])
        .output()
        .expect("browser screen orientation example should run");

    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/browser_screen_orientation.stdout")
    );
}
