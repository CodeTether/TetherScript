pub fn check(stdout: &str, expected_url: &str) {
    super::expect(stdout, "native-browser-text loading clicked mouse below");
    super::expect(stdout, expected_url);
    super::expect(stdout, "native-browser-screenshot png");
    super::expect(stdout, "native-browser-trusted true");
    super::expect(stdout, "native-browser-health false false true");
    super::expect(stdout, "native-browser-wait-more true true");
    super::expect(stdout, "native-browser-snapshots true true");
    super::expect(stdout, "native-browser-query clicked true");
    super::expect(
        stdout,
        &format!("native-browser-history {expected_url}second"),
    );
    super::expect(stdout, "native-browser-wait true");
    super::expect(stdout, "native-browser-network-idle true");
    super::expect(stdout, "native-browser-wait-timeout true");
    super::expect(stdout, "native-browser-stop true");
}
