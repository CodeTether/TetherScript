//! Agent request writing and child cleanup.

use std::io::Write;

use super::AgentClient;

impl AgentClient {
    pub(crate) fn send_prompt(&mut self, prompt: &str) -> Result<(), String> {
        self.request("agent/message", Some(prompt))
    }

    pub(crate) fn request(&mut self, method: &str, prompt: Option<&str>) -> Result<(), String> {
        let request = super::super::protocol::request(self.next_id, method, prompt)?;
        self.next_id += 1;
        writeln!(self.input, "{request}").map_err(|e| format!("agent RPC write failed: {e}"))?;
        self.input
            .flush()
            .map_err(|e| format!("agent RPC flush failed: {e}"))
    }
}

impl Drop for AgentClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
