use tetherscript::{compiler::Compiler, interp::Interpreter, lexer::Lexer, parser::Parser, vm::VM};

fn run_both(source: &str) {
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    Interpreter::new().run(&program).unwrap();
    VM::new().run(Compiler::compile_program(&program)).unwrap();
}

#[test]
fn moved_resources_cross_every_ownership_boundary() {
    run_both(
        r#"
        fn forward(value) { move value }
        fn main() {
            let timer = resource.timer(0).unwrap()
            let forwarded = forward(move timer)
            let values = [move forwarded]
            let task = resource.task().unwrap()
            task.complete(move values).unwrap()
            let received = task.result().unwrap()
            let restored = received.pop()
            restored.cancel().unwrap()
        }
        "#,
    );
}
