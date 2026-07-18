#[path = "browser_native_e2e/assertions.rs"]
mod assertions;
#[path = "browser_native_e2e/fixture.rs"]
mod fixture;
#[path = "browser_native_e2e/fixture_response.rs"]
mod fixture_response;
#[path = "browser_native_e2e/host.rs"]
mod host;
#[path = "browser_native_e2e/ready.rs"]
mod ready;

use std::process::Command;

#[test]
fn tether_script_drives_native_browser_host_end_to_end() {
    let fixture = fixture::Fixture::start();
    let expected_url = fixture.url.clone();
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/browser-e2e");
    std::fs::create_dir_all(&evidence).expect("create browser E2E evidence directory");
    let screenshot = evidence.join("native-browser-e2e.png");
    let upload = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello.tether");
    let mut host = host::NativeHost::start();
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args([
            "run",
            "--grant-browser",
            &host.endpoint,
            "--browser-origin",
            &fixture.url,
            "--browser-scope",
            "all",
            "examples/native_browser_e2e.tether",
            "--",
            &fixture.url,
            screenshot.to_str().unwrap(),
            upload.to_str().unwrap(),
        ])
        .output()
        .expect("run native browser example");
    host.stop();
    fixture.stop();
    assertions::check(
        output,
        &expected_url,
        &screenshot,
        upload.metadata().unwrap().len(),
    );
}
