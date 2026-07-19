use std::process::Command;

#[test]
fn browser_render_example_runs_after_interpolation() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_render.tether"])
        .output()
        .expect("run browser rendering example");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rendered by TetherScript."));
    assert!(stdout.contains("viewport 80x5"));
    assert!(stdout.ends_with("36\n"));
}
