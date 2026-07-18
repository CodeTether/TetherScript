#[path = "browser_native_e2e/fixture.rs"]
mod fixture;
#[path = "browser_native_e2e/fixture_response.rs"]
mod fixture_response;
#[path = "browser_native_e2e/host.rs"]
mod host;
#[path = "browser_agent_e2e/input.rs"]
mod input;
#[path = "browser_native_e2e/ready.rs"]
mod ready;
#[path = "browser_agent_e2e/runner.rs"]
mod runner;

#[test]
fn agent_tui_drives_persistent_native_browser_tools() {
    let fixture = fixture::Fixture::start();
    let mut host = host::NativeHost::start();
    let output = runner::run(&host.endpoint, &fixture.url);
    host.stop();
    fixture.stop();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches(r#""isError":false"#).count(), 4, "{stdout}");
    assert!(stdout.contains("clicked"), "{stdout}");
}
