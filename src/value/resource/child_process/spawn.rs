//! Child spawning policy.

use std::process::{Child, Command, Stdio};

pub(super) fn child(command: &str, args: &[String]) -> Result<Child, String> {
    Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("resource.child_process `{command}`: {error}"))
}
