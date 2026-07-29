//! Git evidence and user-selected validation command execution.

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;

use std::process::Command;

use eframe::egui;

use super::job::Job;

pub(super) fn refresh(job: &mut Job, context: &egui::Context) {
    job.start("Working tree evidence", context, || {
        let status = run("git", &["status", "--short"])?;
        let diff = run("git", &["diff", "--no-ext-diff", "--"])?;
        Ok(format!("GIT STATUS\n{status}\n\nUNIFIED DIFF\n{diff}"))
    });
}

pub(super) fn validate(job: &mut Job, command: &str, context: &egui::Context) {
    let command = command.to_string();
    let label = format!("Validation: {command}");
    job.start(&label, context, move || shell(&command));
}

fn shell(command: &str) -> Result<String, String> {
    #[cfg(windows)]
    let output = Command::new("cmd").args(["/C", command]).output();
    #[cfg(not(windows))]
    let output = Command::new("sh").args(["-lc", command]).output();
    describe(output.map_err(|error| error.to_string())?)
}

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    describe(
        Command::new(program)
            .args(args)
            .output()
            .map_err(|error| error.to_string())?,
    )
}

fn describe(output: std::process::Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!("exit: {}\n{stdout}{stderr}", output.status);
    if output.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}
