use std::path::Path;
use std::process::Output;

pub fn check(output: Output, expected_url: &str, screenshot: &Path, upload_size: u64) {
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
            .any(|line| line == "native-browser-text clicked below"),
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
    assert!(stdout.contains("native-browser-type ABC"), "{stdout}");
    assert!(
        stdout.contains("native-browser-key-events dAiuAdBiuBdCiuCdDiuDdEiuE"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-keyboard-type ABCDE"),
        "{stdout}"
    );
    assert!(stdout.contains("native-browser-focus blurred"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "native-browser-upload hello.tether:text/plain:{upload_size}:ic"
        )),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-toggle true false:cihcih"),
        "{stdout}"
    );
    let selector_scroll = line_value(&stdout, "native-browser-selector-scroll");
    assert!(selector_scroll.parse::<i64>().unwrap() > 0, "{stdout}");
    assert!(
        stdout.contains("native-browser-coordinate-scroll 7,11"),
        "{stdout}"
    );
    assert!(stdout.contains("native-browser-wait true"), "{stdout}");
    assert!(
        stdout.contains("native-browser-wait-timeout true"),
        "{stdout}"
    );
    let png = std::fs::read(screenshot).expect("native screenshot exists");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}

fn line_value<'a>(stdout: &'a str, label: &str) -> &'a str {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{label} ")))
        .unwrap()
}
