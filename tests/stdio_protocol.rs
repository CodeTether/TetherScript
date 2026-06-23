use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn stdio_agent_keeps_jsonrpc_on_stdout() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/agent_tui.tether"])
        .env("TETHERSCRIPT_AGENT_MODE", "rpc")
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
    assert!(stdout.contains("tetherscript"));
    assert!(!stdout.contains("tetherscript stdio agent"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("tetherscript stdio agent"));
}

#[test]
fn stdio_agent_tools_can_edit_workspace() {
    let dir = temp_dir("stdio-edit");
    std::fs::create_dir_all(&dir).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("agent_tui.tether");
    let command = if cfg!(windows) {
        "Get-Content note.txt"
    } else {
        "cat note.txt"
    };
    let input = edit_input(command);
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", script.to_str().unwrap()])
        .env("TETHERSCRIPT_AGENT_MODE", "rpc")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript should spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("improved"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("tetherscript stdio agent"));
}

fn input() -> &'static [u8] {
    br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"cwd","arguments":{}}}
"#
}

fn edit_input(command: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"write\",\"arguments\":{{\"path\":\"note.txt\",\"body\":\"improved\"}}}}}}\n\
         {{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{{\"name\":\"read\",\"arguments\":{{\"path\":\"note.txt\"}}}}}}\n\
         {{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{{\"name\":\"run\",\"arguments\":{{\"command\":\"{}\"}}}}}}\n",
        command
    )
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("tetherscript-{label}-{nanos}"))
}
