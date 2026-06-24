use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn agent_tui_runs_interactive_stdio_tui_by_default() {
    let mut child = agent_command()
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
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "agent tui failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tetherscript agent tui"));
    assert!(stdout.contains("[tool] cwd:"));
    assert!(stdout.contains("+ done"));
}

#[test]
fn agent_tui_exits_on_stdin_eof_instead_of_redrawing_forever() {
    let child = agent_command()
        .args(["run", "examples/agent_tui.tether"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript binary should spawn");
    let output = wait_with_timeout(child, Duration::from_secs(2));
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tui_read_event: end of input"));
    assert!(stdout.contains("+ done"));
}

#[test]
fn agent_tui_prompt_without_provider_stays_in_tui() {
    let mut child = agent_command()
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
        .write_all(b"hi\n/quit\n")
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[agent] provider: provider capability missing"));
    assert!(stdout.contains("+ done"));
}

#[test]
fn agent_tui_empty_multiline_send_returns_to_waiting() {
    let mut child = agent_command()
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
        .write_all(b"/multi\n/send\n/quit\n")
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[system] input: empty multiline ignored"));
    assert!(stdout.contains("+ done"));
}

#[test]
fn agent_tui_plain_answer_uses_one_provider_request() {
    let (addr, handle) = spawn_provider_with(vec![final_body()]);
    let mut child = agent_command()
        .args([
            "run",
            "--grant-provider",
            &format!("http://{addr}"),
            "examples/agent_tui.tether",
        ])
        .env("TETHERSCRIPT_AGENT_MODEL", "test-model")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript binary should spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"hello\n/quit\n")
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let requests = handle.join().expect("provider thread should finish");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(requests.len(), 1);
    assert!(String::from_utf8_lossy(&output.stdout).contains("[agent] provider: done"));
}

#[test]
fn agent_tui_sends_tools_and_executes_model_tool_call() {
    let (addr, handle) = spawn_provider();
    let mut child = agent_command()
        .args([
            "run",
            "--grant-provider",
            &format!("http://{addr}"),
            "examples/agent_tui.tether",
        ])
        .env("TETHERSCRIPT_AGENT_MODEL", "test-model")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript binary should spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"where am i?\n/quit\n")
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let requests = handle.join().expect("provider thread should finish");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tetherscript agent tui"));
    assert!(stdout.contains("+ working"));
    assert!(stdout.contains("[tool] cwd:"));
    assert!(stdout.contains("[agent] provider: done"));
    assert!(requests[0].contains("\"tools\""));
    assert!(requests[0].contains("\"name\":\"cwd\""));
    assert!(requests[1].contains("\"tool_call_id\":\"call-1\""));
}

#[test]
fn agent_tui_restarts_after_http_tool_edits_script() {
    let root = temp_root("agent-tui-reload");
    std::fs::create_dir_all(&root).unwrap();
    let script = root.join("agent_tui_reload.tether");
    std::fs::copy("examples/agent_tui.tether", &script).unwrap();
    let script_text = script.to_string_lossy().into_owned();
    let (addr, handle) = spawn_provider_with(vec![
        append_call_body(&script_text),
        final_body().to_string(),
    ]);
    let mut child = agent_command()
        .current_dir(&root)
        .arg("run")
        .arg("--grant-provider")
        .arg(format!("http://{addr}"))
        .arg(&script)
        .env("TETHERSCRIPT_AGENT_MODEL", "test-model")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tetherscript binary should spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"edit yourself\n/quit\n")
        .unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let requests = handle.join().expect("provider thread should finish");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[system] reload: source changed; restarting"));
    assert!(stdout.contains("+ done"));
    assert_eq!(requests.len(), 2);
    assert!(!root.join(".tetherscript").join("reload").exists());
    let _ = std::fs::remove_dir_all(root);
}

fn agent_command() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tetherscript"));
    cmd.env("CODETETHER_DISABLE_ENV_FALLBACK", "1")
        .env_remove("TETHERSCRIPT_PROVIDER")
        .env_remove("TETHERSCRIPT_AGENT_PROVIDER")
        .env_remove("TETHERSCRIPT_PROVIDER_ENDPOINT")
        .env_remove("VAULT_ADDR")
        .env_remove("VAULT_TOKEN");
    cmd
}

fn wait_with_timeout(mut child: Child, timeout: Duration) -> Output {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();
    panic!(
        "agent tui did not exit before timeout; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn spawn_provider() -> (String, thread::JoinHandle<Vec<String>>) {
    spawn_provider_with(vec![tool_call_body(), final_body()])
}

fn spawn_provider_with<T: Into<String>>(
    bodies: Vec<T>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let bodies: Vec<String> = bodies.into_iter().map(Into::into).collect();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while requests.len() < bodies.len() && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    requests.push(read_request(&mut stream));
                    write_response(&mut stream, &bodies[requests.len() - 1]);
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("provider accept failed: {err}"),
            }
        }
        requests
    });
    (addr, handle)
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut buf = [0u8; 512];
    while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = stream.read(&mut buf).unwrap();
        bytes.extend_from_slice(&buf[..n]);
    }
    let header_len = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    let len = content_len(&String::from_utf8_lossy(&bytes[..header_len]));
    while bytes.len() < header_len + len {
        let n = stream.read(&mut buf).unwrap();
        bytes.extend_from_slice(&buf[..n]);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn content_len(head: &str) -> usize {
    head.lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

fn write_response(stream: &mut TcpStream, body: &str) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .unwrap();
}

fn tool_call_body() -> &'static str {
    r#"{"choices":[{"message":{"role":"assistant","content":"","tool_calls":[{"id":"call-1","type":"function","function":{"name":"cwd","arguments":"{}"}}]}}]}"#
}

fn final_body() -> &'static str {
    r#"{"choices":[{"message":{"role":"assistant","content":"done"}}]}"#
}

fn append_call_body(path: &str) -> String {
    let args = format!(
        "{{\"path\":{},\"body\":\"\\n// hot reload smoke\\n\"}}",
        json_string(path)
    );
    format!(
        "{{\"choices\":[{{\"message\":{{\"role\":\"assistant\",\"content\":\"\",\"tool_calls\":[{{\"id\":\"call-append\",\"type\":\"function\",\"function\":{{\"name\":\"append\",\"arguments\":{}}}}}]}}}}]}}",
        json_string(&args)
    )
}

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn temp_root(name: &str) -> PathBuf {
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{tick}", std::process::id()))
}
