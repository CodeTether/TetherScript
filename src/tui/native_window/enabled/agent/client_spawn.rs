//! Agent RPC subprocess creation and stream wiring.

use std::process::{Command, Stdio};
use std::sync::mpsc;

use eframe::egui;

use super::{client_output, AgentClient};

impl AgentClient {
    pub fn spawn(path: &str, context: egui::Context) -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .args(["run", path])
            .env("TETHERSCRIPT_AGENT_MODE", "rpc")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("agent RPC launch failed: {e}"))?;
        let input = child.stdin.take().ok_or("agent RPC stdin unavailable")?;
        let output = child.stdout.take().ok_or("agent RPC stdout unavailable")?;
        let errors = child.stderr.take().ok_or("agent RPC stderr unavailable")?;
        let (sender, receiver) = mpsc::channel();
        client_output::forward(
            output,
            sender.clone(),
            context.clone(),
            client_output::Output::Line,
        );
        client_output::forward(errors, sender, context, client_output::Output::Error);
        let mut client = Self {
            child,
            input,
            output: receiver,
            next_id: 1,
        };
        client.request("agent/state", None)?;
        Ok(client)
    }
}
