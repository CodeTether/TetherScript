use std::process::Command;

#[cfg(windows)]
pub(super) fn list() -> Result<String, String> {
    output(
        Command::new("tasklist").args(["/FO", "CSV", "/NH"]),
        "process_list",
    )
}

#[cfg(not(windows))]
pub(super) fn list() -> Result<String, String> {
    output(
        Command::new("ps").args(["-e", "-o", "pid=,comm="]),
        "process_list",
    )
}

#[cfg(windows)]
pub(super) fn kill(pid: i64, force: bool) -> Result<(), String> {
    let mut command = Command::new("taskkill");
    command.args(["/PID", &pid.to_string(), "/T"]);
    if force {
        command.arg("/F");
    }
    output(&mut command, "process_kill").map(|_| ())
}

#[cfg(not(windows))]
pub(super) fn kill(pid: i64, force: bool) -> Result<(), String> {
    let signal = if force { "-KILL" } else { "-TERM" };
    output(
        Command::new("kill").args([signal, &pid.to_string()]),
        "process_kill",
    )
    .map(|_| ())
}

fn output(command: &mut Command, label: &str) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|error| format!("{label}: launch failed: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!(
            "{label}: operating system rejected request: {detail}"
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
