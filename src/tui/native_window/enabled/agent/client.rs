//! JSON-RPC subprocess bridge to the script-owned agent.

#[path = "client_output.rs"]
pub(super) mod output;
#[path = "client_request.rs"]
mod request;

use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::Receiver;

pub(super) struct AgentClient {
    child: Child,
    pub(super) input: ChildStdin,
    pub(super) output: Receiver<output::Output>,
    pub(super) next_id: i64,
}

impl AgentClient {
    pub(super) fn spawn(path: &str) -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .args(["run", path])
            .env("TETHERSCRIPT_AGENT_MODE", "rpc")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("agent RPC launch failed: {e}"))?;
        let input = child.stdin.take().ok_or("agent RPC stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("agent RPC stdout unavailable")?;
        let stderr = child.stderr.take().ok_or("agent RPC stderr unavailable")?;
        let (sender, output) = std::sync::mpsc::channel();
        self::output::forward(stdout, sender.clone(), self::output::Output::Line);
        self::output::forward(stderr, sender, self::output::Output::Error);
        let mut client = Self {
            child,
            input,
            output,
            next_id: 1,
        };
        client.request("agent/state", None)?;
        Ok(client)
    }
}
