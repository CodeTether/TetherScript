use std::process::Command;

#[test]
fn standalone_browser_eval_css_media_gap_is_retained() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_css_media.tether"])
        .output()
        .expect("browser CSS media example should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        include_str!("examples/browser_css_media.stdout")
    );
}
