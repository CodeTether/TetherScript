//! Startup path for executables with embedded tetherscript source.

use crate::main_caps::RunCaps;
use crate::{ownership, Compiler, Lexer, Parser, VM};

#[path = "main_embedded_args.rs"]
mod args;
#[path = "main_embedded_marker.rs"]
mod marker;
#[path = "main_embedded_reload.rs"]
mod reload;

pub(crate) fn run(src: &str, process_args: &[String]) -> Result<(), String> {
    reload::run(src, process_args)
}

pub(super) fn run_source(src: &str, opts: &args::EmbeddedArgs) -> Result<(), String> {
    let mut opts = opts.clone();
    opts.full_access = crate::main_caps::script_full_access(src, opts.full_access);
    let program = parse(src)?;
    if let Err(diagnostics) = ownership::analyze(&program) {
        for diagnostic in diagnostics {
            eprintln!("tetherscript: ownership error: {}", diagnostic.message);
        }
        return Err("ownership analysis failed".into());
    }
    let chunk = Compiler::compile_program(&program);
    let mut vm = VM::new();
    vm.install_cli_args(&opts.script_args);
    let browser_origins = Vec::new();
    let browser_scopes = Vec::new();
    let caps = RunCaps {
        fs_grant: &opts.fs_grant,
        full_access: opts.full_access,
        provider_grant: &opts.provider_grant,
        provider_key: &opts.provider_key,
        provider_vault: &opts.provider_vault,
        rpc_grant: &opts.rpc_grant,
        browser_grant: &None,
        browser_origins: &browser_origins,
        browser_scopes: &browser_scopes,
    };
    crate::main_caps::grant_vm(&mut vm, &caps)?;
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
