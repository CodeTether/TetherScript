use std::process::Command;

use tetherscript::compiler::Compiler;
use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;
use tetherscript::vm::VM;

fn assert_example(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(args)
        .output()
        .expect("owned resource example should start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("examples/owned_resources.stdout")
    );
}

#[test]
fn resource_example_matches_vm_golden_output() {
    // The example binds a TCP listener, which now requires an explicit grant.
    assert_example(&[
        "run",
        "--grant-tcp",
        "127.0.0.1",
        "examples/owned_resources.tether",
    ]);
}

#[test]
fn resource_example_matches_interpreter_golden_output() {
    assert_example(&[
        "run",
        "--interp",
        "--grant-tcp",
        "127.0.0.1",
        "examples/owned_resources.tether",
    ]);
}

/// The example must fail without the grant, or the gate is not doing anything.
#[test]
fn resource_example_is_denied_without_a_tcp_grant() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/owned_resources.tether"])
        .output()
        .expect("example should start");

    assert!(!output.status.success(), "expected a capability denial");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--grant-tcp"), "got: {stderr}");
}

#[test]
fn resource_move_leaves_a_runtime_tombstone() {
    let source = "fn main() { let ch = resource.channel(1).unwrap() let moved = move ch ch.len() }";
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let error = VM::new()
        .run(Compiler::compile_program(&program))
        .unwrap_err();
    assert!(error.contains("use of moved value `ch`"), "{error}");
}
