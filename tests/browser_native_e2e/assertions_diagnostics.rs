pub fn check(stdout: &str) {
    super::expect(
        stdout,
        "native-browser-diagnostics true true true true true div true",
    );
    super::expect(
        stdout,
        "native-browser-react-more 18.3.0 true true true true ready useState Root>App",
    );
    super::expect(stdout, "native-browser-runtime-diagnostics 0 ");
    super::expect(stdout, "native-browser-indexed-db 0");
    super::expect(stdout, "native-browser-session-storage 0");
    super::expect(stdout, "native-browser-storage-clear 8 1 0 0");
}
