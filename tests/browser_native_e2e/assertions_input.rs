pub fn check(stdout: &str, upload_size: u64) {
    super::expect(stdout, "native-browser-keyboard A");
    super::expect(stdout, "native-browser-fill filled");
    super::expect(stdout, "native-browser-type ABC");
    super::expect(
        stdout,
        "native-browser-key-events dAiuAdBiuBdCiuCdDiuDdEiuEidEnteruEnter",
    );
    super::expect(stdout, "native-browser-keyboard-type ABCDE");
    super::expect(stdout, "native-browser-fill-native native");
    super::expect(stdout, "native-browser-keyboard-press press");
    super::expect(stdout, "native-browser-focus blurred");
    super::expect(
        stdout,
        &format!("native-browser-upload hello.tether:text/plain:{upload_size}:ic"),
    );
    super::expect(stdout, "native-browser-toggle true false:cihcih");
    super::expect(stdout, "native-browser-mouse true:");
    super::expect(stdout, "native-browser-tabs 2 1");
    let scroll = super::line_value(stdout, "native-browser-selector-scroll");
    assert!(scroll.parse::<i64>().unwrap() > 0, "{stdout}");
    super::expect(stdout, "native-browser-coordinate-scroll 7,11");
}
