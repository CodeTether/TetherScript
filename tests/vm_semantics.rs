use tetherscript::compiler::Compiler;
use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;
use tetherscript::vm::VM;

fn run_vm(source: &str) -> Result<(), String> {
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    VM::new().run(Compiler::compile_program(&program))
}

#[test]
fn function_params_are_env_bindings() {
    let source = "fn id(x) { return x } fn main() { assert(id(41) + 1 == 42, \"param\") }";

    run_vm(source).unwrap();
}

#[test]
fn moved_function_local_reports_use_after_move() {
    let source = "fn main() { let xs = [1] let ys = move xs ys.len() xs.len() }";

    let err = run_vm(source).unwrap_err();

    assert!(err.contains("use of moved value `xs`"), "{err}");
}

#[test]
fn immutable_function_local_assignment_is_rejected() {
    let source = "fn main() { let x = 1 x = 2 }";

    let err = run_vm(source).unwrap_err();

    assert!(
        err.contains("cannot assign to immutable binding `x`"),
        "{err}"
    );
}

#[test]
fn returning_from_inside_a_for_loop_leaves_no_operands_behind() {
    // The loop keeps its iterable and index on the operand stack. A `return` from
    // inside the body used to pop only the return value, so the loop state survived
    // the call and was misread as the caller's next operand — a method call on the
    // result then failed with "int is not callable", naming neither the loop nor the
    // function that leaked it.
    let source = "fn first(xs) { for x in xs { return Ok(x) } return Err(\"empty\") } \
                  fn main() { assert(first([7]).is_ok(), \"chained call after loop return\") }";

    run_vm(source).unwrap();
}

#[test]
fn returning_from_inside_a_while_loop_leaves_no_operands_behind() {
    let source =
        "fn find(limit) { let mut i = 0 while i < limit { return Ok(i) } return Err(\"none\") } \
                  fn main() { assert(find(3).is_ok(), \"chained call after while return\") }";

    run_vm(source).unwrap();
}

#[test]
fn a_loop_return_value_survives_being_chained_repeatedly() {
    // Each abandoned frame must clear its own operands, not just the innermost one.
    let source = "fn inner(xs) { for x in xs { return Ok(x) } return Err(\"empty\") } \
                  fn outer(xs) { for x in xs { return inner(xs) } return Err(\"empty\") } \
                  fn main() { \
                      assert(outer([1, 2]).is_ok(), \"nested\") \
                      assert(inner([1]).is_ok(), \"first\") \
                      assert(inner([2]).is_ok(), \"second\") \
                  }";

    run_vm(source).unwrap();
}
