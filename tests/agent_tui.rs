use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn agent_tui_runs_tool_call_from_terminal_loop() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/agent_tui.tether"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript binary should spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"/tool cwd\n/quit\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "agent tui failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[tool] cwd:"));
    assert!(stdout.contains("tetherscript agent tui"));
}
