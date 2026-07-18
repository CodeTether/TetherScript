use std::process::Command;

#[test]
fn browser_canvas_image_data_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_canvas_image_data.tether"])
        .output()
        .expect("browser Canvas ImageData example should run");

    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/browser_canvas_image_data.stdout")
    );
}
