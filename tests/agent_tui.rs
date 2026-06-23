use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn agent_tui_runs_interactive_stdio_tui_by_default() {
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
fn agent_tui_prompt_without_provider_stays_in_tui() {
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
fn agent_tui_sends_tools_and_executes_model_tool_call() {
    let (addr, handle) = spawn_provider();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
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
    assert!(stdout.contains("[tool] cwd:"));
    assert!(stdout.contains("[agent] provider: done"));
    assert!(requests[0].contains("\"tools\""));
    assert!(requests[0].contains("\"name\":\"cwd\""));
    assert!(requests[1].contains("\"tool_call_id\":\"call-1\""));
}

fn spawn_provider() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while requests.len() < 2 && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    requests.push(read_request(&mut stream));
                    let body = if requests.len() == 1 {
                        tool_call_body()
                    } else {
                        final_body()
                    };
                    write_response(&mut stream, body);
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
