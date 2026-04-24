//! tetherscript — a dynamically-typed scripting language with Rust-style ownership.
//!
//! Usage:
//!   tetherscript <file.kl>                  # run (tree-walking interpreter)
//!   tetherscript --vm       <file.kl>       # run (bytecode VM)
//!   tetherscript --tokens   <file.kl>       # dump tokens and exit
//!   tetherscript --ast      <file.kl>       # dump AST and exit
//!   tetherscript --bytecode <file.kl>       # dump compiled bytecode and exit
//!   tetherscript --lsp                      # serve LSP over stdio

mod ast;
mod bytecode;
mod capability;
mod compiler;
mod http;
mod interp;
mod json;
mod lexer;
mod lsp;
mod output;
mod parser;
mod smtp;
mod system;
mod token;
mod value;
mod vm;

use std::env;
use std::fs;
use std::process;

use compiler::Compiler;
use interp::Interpreter;
use lexer::Lexer;
use parser::Parser;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: tetherscript [--tokens|--ast|--bytecode|--vm|--lsp] <file.kl>");
        process::exit(2);
    }

    if args[1] == "--lsp" {
        if let Err(e) = lsp::run() {
            eprintln!("tetherscript-lsp: {}", e);
            process::exit(1);
        }
        return;
    }

    let (mode, path) = match args[1].as_str() {
        "--tokens" if args.len() >= 3 => ("tokens", &args[2]),
        "--ast" if args.len() >= 3 => ("ast", &args[2]),
        "--bytecode" if args.len() >= 3 => ("bytecode", &args[2]),
        "--vm" if args.len() >= 3 => ("vm", &args[2]),
        _ => ("run", &args[1]),
    };

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tetherscript: can't read {}: {}", path, e);
            process::exit(1);
        }
    };

    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tetherscript: lex error at {}:{}: {}", e.line, e.col, e.msg);
            process::exit(1);
        }
    };

    if mode == "tokens" {
        for t in &tokens {
            println!("{:>3}:{:<3}  {:?}", t.line, t.col, t.token);
        }
        return;
    }

    let program = match Parser::new(tokens).parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "tetherscript: parse error at {}:{}: {}",
                e.line, e.col, e.msg
            );
            process::exit(1);
        }
    };

    if mode == "ast" {
        println!("{:#?}", program);
        return;
    }

    if mode == "bytecode" {
        let chunk = Compiler::compile_program(&program);
        println!("{:#?}", chunk);
        return;
    }

    if mode == "vm" {
        let chunk = Compiler::compile_program(&program);
        let mut vm = VM::new();
        if let Err(e) = vm.run(chunk) {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
        }
        return;
    }

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("tetherscript: {}", e);
        process::exit(1);
    }
}
