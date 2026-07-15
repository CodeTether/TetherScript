use crate::{bytecode_visual, compiler::Compiler, ir, token::Spanned};

use super::{args::Mode, frontend};

pub(super) fn render(mode: Mode, tokens: Vec<Spanned>) -> Result<String, String> {
    if mode == Mode::Tokens {
        return Ok(tokens
            .iter()
            .map(|token| format!("{:>3}:{:<3}  {:?}", token.line, token.col, token.token))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    let program = frontend::program(tokens)?;
    match mode {
        Mode::Ast => Ok(format!("{program:#?}")),
        Mode::Bytecode => Ok(format!("{:#?}", Compiler::compile_program(&program))),
        Mode::BytecodeVisual => {
            let chunk = Compiler::compile_program(&program);
            Ok(bytecode_visual::render(&chunk))
        }
        Mode::Ir => {
            let module = ir::lower_program(&program).map_err(lower_error)?;
            ir::verify(&module).map_err(verify_error)?;
            Ok(ir::render(&module))
        }
        Mode::Tokens => unreachable!(),
    }
}

fn lower_error(error: ir::LowerError) -> String {
    error.to_string()
}

fn verify_error(error: ir::VerifyError) -> String {
    error.to_string()
}
