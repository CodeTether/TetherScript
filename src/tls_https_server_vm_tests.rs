//! VM registration coverage for the HTTPS listener built-in.

use crate::compiler::Compiler;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::VM;

#[test]
fn vm_exposes_https_server_with_pem_validation() {
    let source = r#"
fn main() {
    https_serve(443, "bad cert", "bad key", fn(request) { request })
}
"#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let error = VM::new()
        .run(Compiler::compile_program(&program))
        .unwrap_err();
    assert!(error.contains("https_serve: invalid TLS identity"));
}
