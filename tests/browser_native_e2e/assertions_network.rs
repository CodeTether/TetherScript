pub fn check(stdout: &str) {
    let network = super::line_value(stdout, "native-browser-network");
    assert!(network.parse::<usize>().unwrap() > 0, "{stdout}");
    super::expect(stdout, "native-browser-network-har true");
    super::expect(stdout, "native-browser-network-waits 1 1");
    super::expect(stdout, "native-browser-requests 200 200 200 200");
    super::expect(stdout, "native-browser-network-more 0 200");
}
