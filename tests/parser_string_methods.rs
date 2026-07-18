use tetherscript::ast::{Expr, Stmt};
use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;

#[test]
fn quoted_member_name_parses_as_method_call() {
    let tokens = Lexer::new("fn main() { browser.\"react.detect\"() }")
        .tokenize()
        .unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let Stmt::FnDecl { body, .. } = &program.stmts[0] else {
        panic!("expected function declaration")
    };
    let Stmt::Expr { expr, .. } = &body.stmts[0] else {
        panic!("expected method expression")
    };
    let Expr::Method { target, name, args } = expr else {
        panic!("expected method call")
    };

    assert!(matches!(target.as_ref(), Expr::Ident(value) if value == "browser"));
    assert_eq!(name, "react.detect");
    assert!(args.is_empty());
}
