use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn stdio_mcp_tui_keeps_jsonrpc_on_stdout() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/stdio_mcp_tui.tether"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript should spawn");
    child.stdin.as_mut().unwrap().write_all(input()).unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"serverInfo\""));
    assert!(stdout.contains("\"id\":2"));
    assert!(stdout.contains("\"tools\""));
    assert!(stdout.contains("\"id\":3"));
    assert!(stdout.contains("\"pong\""));
    assert!(!stdout.contains("stdio mcp tui"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("stdio mcp tui"));
}

fn input() -> &'static [u8] {
    br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping","arguments":{}}}
"#
}
