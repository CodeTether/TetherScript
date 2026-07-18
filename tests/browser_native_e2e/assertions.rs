#[path = "assertions_core.rs"]
mod core;
#[path = "assertions_diagnostics.rs"]
mod diagnostics;
#[path = "assertions_input.rs"]
mod input;
#[path = "assertions_network.rs"]
mod network;
#[path = "assertions_visual.rs"]
mod visual;

use std::path::Path;
use std::process::Output;

pub fn check(output: Output, expected_url: &str, screenshot: &Path, upload_size: u64) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    std::fs::write(screenshot.with_extension("stdout"), stdout.as_bytes()).unwrap();
    std::fs::write(screenshot.with_extension("stderr"), stderr.as_bytes()).unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        stderr
    );
    core::check(&stdout, expected_url);
    input::check(&stdout, upload_size);
    network::check(&stdout);
    diagnostics::check(&stdout);
    visual::check(&stdout);
    let png = std::fs::read(screenshot).expect("native screenshot exists");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}

pub(super) fn expect(stdout: &str, text: &str) {
    assert!(stdout.contains(text), "missing `{text}` in:\n{stdout}");
}

pub(super) fn line_value<'a>(stdout: &'a str, label: &str) -> &'a str {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{label} ")))
        .unwrap()
}
