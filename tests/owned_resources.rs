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
    assert_example(&["run", "examples/owned_resources.tether"]);
}

#[test]
fn resource_example_matches_interpreter_golden_output() {
    assert_example(&["run", "--interp", "examples/owned_resources.tether"]);
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
