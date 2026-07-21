use std::process::Command;

fn assert_example(engine: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(engine)
        .arg("examples/render_surface.tether")
        .output()
        .expect("render-surface example should start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/render_surface.stdout")
    );
}

#[test]
fn render_surface_matches_vm_golden_output() {
    assert_example(&["run"]);
}

#[test]
fn render_surface_matches_interpreter_golden_output() {
    assert_example(&["run", "--interp"]);
}
