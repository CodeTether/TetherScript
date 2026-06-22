use std::process::Command;

#[test]
fn tui_agent_panel_renders_with_vm() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/tui_agent_panel.tether"])
        .output()
        .expect("tetherscript binary should run");
    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[agent] planner: mapped work into a frame"));
    assert!(stdout.contains("+ agent bus"));
}
