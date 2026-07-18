use std::io::Write;
use std::process::{Command, Output, Stdio};

pub fn run(endpoint: &str, url: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args([
            "run",
            "--grant-browser",
            endpoint,
            "--browser-origin",
            url,
            "--browser-scope",
            "all",
            "examples/agent_tui.tether",
        ])
        .env("TETHERSCRIPT_AGENT_MODE", "rpc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(super::input::requests(url).as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
