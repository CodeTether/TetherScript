use std::process::Command;

fn assert_example(engine: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(engine)
        .arg("examples/async_basic.tether")
        .output()
        .expect("async example should start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/async_basic.stdout")
    );
}

#[test]
fn cooperative_tasks_match_vm_golden_output() {
    assert_example(&["run"]);
}

#[test]
fn cooperative_tasks_match_interpreter_golden_output() {
    assert_example(&["run", "--interp"]);
}
