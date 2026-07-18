pub fn check(stdout: &str) {
    super::expect(stdout, "native-browser-visual true true 0 true true");
    super::expect(stdout, "native-browser-visual-more true true true true");
    super::expect(stdout, "native-browser-trace true true true true true true");
}
