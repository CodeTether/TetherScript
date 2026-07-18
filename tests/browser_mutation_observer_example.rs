use std::process::Command;

#[test]
fn browser_mutation_observer_example_matches_golden_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/browser_mutation_observer.tether"])
        .output()
        .expect("browser MutationObserver example should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        include_str!("examples/browser_mutation_observer.stdout")
    );
}
