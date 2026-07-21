use tetherscript::{compiler::Compiler, interp::Interpreter, lexer::Lexer, parser::Parser, vm::VM};

fn engine_errors(source: &str) -> [String; 2] {
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let interp = Interpreter::new().run(&program).unwrap_err();
    let vm = VM::new()
        .run(Compiler::compile_program(&program))
        .unwrap_err();
    [interp, vm]
}

fn assert_rejected(source: &str, expected: &str) {
    for error in engine_errors(source) {
        assert!(
            error.contains(expected),
            "expected {expected:?} in {error:?}"
        );
        assert!(error.contains("use `move`"), "{error}");
    }
}

#[test]
fn persistent_sinks_reject_borrowed_resources() {
    let cases = [
        ("let alias=timer", "binding `alias`"),
        ("let mut target=nil target=timer", "assignment"),
        ("let values=[timer]", "list literal"),
        ("let values=[nil] values[0]=timer", "assignment"),
        ("let values=map() values.timer=timer", "assignment"),
        ("let values=[] values.push(timer)", "list.push"),
        ("let value=Ok(timer)", "Ok"),
    ];
    for (sink, expected) in cases {
        let source = format!("fn main() {{ let timer=resource.timer(0).unwrap() {sink} }}");
        assert_rejected(&source, expected);
    }
}

#[test]
fn returns_and_nested_owners_reject_borrows() {
    assert_rejected(
        "fn leak(value){ value } fn main(){ let timer=resource.timer(0).unwrap() leak(timer) }",
        "function return",
    );
    assert_rejected(
        "fn main(){ let timer=resource.timer(0).unwrap() let values=[move timer] let alias=values }",
        "list containing timer resource",
    );
}
