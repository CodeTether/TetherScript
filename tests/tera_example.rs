use std::process::Command;

#[test]
fn tera_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/tera.tether"])
        .output()
        .expect("tera example should run");
    assert!(
        output.status.success(),
        "tera example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/tera.stdout")
    );
}
