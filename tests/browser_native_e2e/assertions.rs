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
            .any(|line| line == "native-browser-text clicked mouse below"),
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
    assert!(
        stdout.contains("native-browser-fill-native native"),
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
    assert!(stdout.contains("native-browser-mouse true:"), "{stdout}");
    assert!(stdout.contains("native-browser-tabs 2 1"), "{stdout}");
    let selector_scroll = line_value(&stdout, "native-browser-selector-scroll");
    assert!(selector_scroll.parse::<i64>().unwrap() > 0, "{stdout}");
    assert!(
        stdout.contains("native-browser-coordinate-scroll 7,11"),
        "{stdout}"
    );
    assert!(stdout.contains("native-browser-wait true"), "{stdout}");
    assert!(
        stdout.contains("native-browser-network-idle true"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-wait-timeout true"),
        "{stdout}"
    );
    let network = line_value(&stdout, "native-browser-network");
    assert!(network.parse::<usize>().unwrap() > 0, "{stdout}");
    assert!(
        stdout.contains("native-browser-network-har true"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-network-waits 1 1"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-requests 200 200 200 200"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-diagnostics 2 1 1 true 1 div true"),
        "{stdout}"
    );
    assert!(stdout.contains("native-browser-indexed-db 0"), "{stdout}");
    assert!(
        stdout.contains("native-browser-storage-clear 8 1 0 0"),
        "{stdout}"
    );
    assert!(
        stdout.contains("native-browser-visual true true 0 true"),
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
