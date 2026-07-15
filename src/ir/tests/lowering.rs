use crate::{ir, lexer::Lexer, parser::Parser};

#[test]
fn lowers_arithmetic_and_named_calls() {
    let source = include_str!("../../../examples/ir_arithmetic.tether");
    let tokens = Lexer::new(source).tokenize().expect("lex IR example");
    let program = Parser::new(tokens)
        .parse_program()
        .expect("parse IR example");
    let module = ir::lower_program(&program).expect("lower IR example");
    ir::verify(&module).expect("verify lowered IR");
    assert_eq!(ir::render(&module), include_str!("expected.tir").trim_end());
}

#[test]
fn rejects_control_flow_until_blocks_are_supported() {
    let source = "fn choose(flag) { if flag { 1 } else { 0 } }";
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let error = ir::lower_program(&program).unwrap_err();
    assert_eq!(
        error.to_string(),
        "cannot lower function `choose`: unsupported expression `if`"
    );
}
