use std::path::Path;
use std::process::Output;

pub fn check(output: Output, expected_url: &str, screenshot: &Path) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    std::fs::write(screenshot.with_extension("stdout"), stdout.as_bytes()).unwrap();
    assert!(
        stdout
            .lines()
            .any(|line| line == "native-browser-text clicked"),
        "{stdout}"
    );
    assert!(stdout.contains(expected_url), "{stdout}");
    assert!(stdout.contains("native-browser-screenshot png"), "{stdout}");
    assert!(stdout.contains("native-browser-trusted true"), "{stdout}");
    assert!(
        stdout.contains(&format!("native-browser-history {expected_url}second")),
        "{stdout}"
    );
    assert!(stdout.contains("native-browser-keyboard A"), "{stdout}");
    assert!(stdout.contains("native-browser-focus blurred"), "{stdout}");
    let png = std::fs::read(screenshot).expect("native screenshot exists");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}
