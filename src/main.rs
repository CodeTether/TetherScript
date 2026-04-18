//! kiln — a dynamically-typed scripting language with Rust-style ownership.
//!
//! Usage:
//!   kiln <file.kl>
//!   kiln --tokens <file.kl>     # dump tokens and exit
//!   kiln --ast    <file.kl>     # dump AST and exit

mod token;
mod lexer;
mod ast;
mod parser;
mod value;
mod interp;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use interp::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: kiln [--tokens|--ast] <file.kl>");
        process::exit(2);
    }

    let (mode, path) = match args[1].as_str() {
        "--tokens" if args.len() >= 3 => ("tokens", &args[2]),
        "--ast"    if args.len() >= 3 => ("ast", &args[2]),
        _                             => ("run", &args[1]),
    };

    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("kiln: can't read {}: {}", path, e);
            process::exit(1);
        }
    };

    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("kiln: lex error at {}:{}: {}", e.line, e.col, e.msg);
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
            eprintln!("kiln: parse error at {}:{}: {}", e.line, e.col, e.msg);
            process::exit(1);
        }
    };

    if mode == "ast" {
        println!("{:#?}", program);
        return;
    }

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("kiln: {}", e);
        process::exit(1);
    }
}
