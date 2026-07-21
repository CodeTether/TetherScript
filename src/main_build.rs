//! `tetherscript build` command.

use std::path::Path;

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let opts = crate::main_build_parse::parse(args)?;
    let source = std::fs::read_to_string(&opts.path)
        .map_err(|e| format!("can't read {}: {e}", opts.path))?;
    let tokens = crate::lexer::Lexer::new(&source)
        .tokenize()
        .map_err(|error| format!("lex error at {}:{}: {}", error.line, error.col, error.msg))?;
    let program = crate::parser::Parser::new(tokens)
        .parse_program()
        .map_err(|error| format!("parse error at {}:{}: {}", error.line, error.col, error.msg))?;
    if !program.imports.is_empty() {
        return Err("standalone builds do not yet bundle imported modules".into());
    }
    crate::embed::write_launcher(&source, Path::new(&opts.output))
}

pub(crate) fn print_help() {
    println!("tetherscript build -- Build a standalone executable launcher");
    println!();
    println!("USAGE:");
    println!("    tetherscript build <file.tether> -o <output>");
    println!();
    println!("The output embeds the script source in a copy of the runner.");
    println!("Process arguments are available from env_args().");
}
