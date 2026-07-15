use std::process::Command;

const EXAMPLE: &str = "examples/ir_arithmetic.tether";

#[test]
fn arithmetic_example_runs_on_existing_vm() {
    let output = command()
        .args(["run", EXAMPLE])
        .output()
        .expect("run example");
    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "42\n");
}

#[test]
fn inspect_ir_matches_textual_artifact() {
    let output = command()
        .args(["inspect", "--ir", EXAMPLE])
        .output()
        .expect("inspect IR");
    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    let expected = include_str!("../src/ir/tests/expected.tir");
    assert_eq!(text(&output.stdout), format!("{}\n", expected.trim_end()));
}

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
