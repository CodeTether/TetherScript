//! Startup path for executables with embedded tetherscript source.

use crate::{ownership, Compiler, Lexer, Parser, VM};

pub(crate) fn run(src: &str, script_args: &[String]) -> Result<(), String> {
    let program = parse(src)?;
    if let Err(diagnostics) = ownership::analyze(&program) {
        for diagnostic in diagnostics {
            eprintln!("tetherscript: ownership error: {}", diagnostic.message);
        }
        return Err("ownership analysis failed".into());
    }
    let chunk = Compiler::compile_program(&program);
    let mut vm = VM::new();
    vm.install_cli_args(script_args);
    vm.run(chunk)
}

fn parse(src: &str) -> Result<crate::ast::Program, String> {
    let tokens = Lexer::new(src)
        .tokenize()
        .map_err(|e| format!("lex error at {}:{}: {}", e.line, e.col, e.msg))?;
    Parser::new(tokens)
        .parse_program()
        .map_err(|e| format!("parse error at {}:{}: {}", e.line, e.col, e.msg))
}
