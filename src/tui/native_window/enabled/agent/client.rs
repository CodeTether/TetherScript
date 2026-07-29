//! JSON-RPC subprocess bridge to `agent_tui.tether`.

#[path = "client_output.rs"]
pub(super) mod client_output;
#[path = "client_spawn.rs"]
mod client_spawn;
#[path = "client_drop.rs"]
mod drop_impl;

use std::io::Write;
use std::process::{Child, ChildStdin};
use std::sync::mpsc::Receiver;

pub(super) struct AgentClient {
    child: Child,
    input: ChildStdin,
    pub(super) output: Receiver<client_output::Output>,
    next_id: i64,
}

impl AgentClient {
    pub fn send_prompt(&mut self, prompt: &str) -> Result<(), String> {
        self.request("agent/message", Some(prompt))
    }

    fn request(&mut self, method: &str, prompt: Option<&str>) -> Result<(), String> {
        let request = super::protocol::request(self.next_id, method, prompt)?;
        self.next_id += 1;
        writeln!(self.input, "{request}").map_err(|e| format!("agent RPC write failed: {e}"))?;
        self.input
            .flush()
            .map_err(|e| format!("agent RPC flush failed: {e}"))
    }
}
