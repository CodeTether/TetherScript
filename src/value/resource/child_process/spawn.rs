//! Child spawning policy.

use std::process::{Child, Command, Stdio};

pub(super) fn child(command: &str, args: &[String]) -> Result<Child, String> {
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("resource.child_process `{command}`: {error}"))
}
