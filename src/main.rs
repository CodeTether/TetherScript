//! rk — a dynamically-typed scripting language with Rust-style ownership.
//!
//! v0.0.1: lexer only. Run with:
//!   cargo run -- examples/hello.rk

mod token;
mod lexer;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: rk <file.rk>");
        process::exit(2);
    }

    let path = &args[1];
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("rk: can't read {}: {}", path, e);
            process::exit(1);
        }
    };

    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("rk: lex error at {}:{}: {}", e.line, e.col, e.msg);
            process::exit(1);
        }
    };

    // For now: just dump tokens. Parser comes next.
    for t in &tokens {
        println!("{:>3}:{:<3}  {:?}", t.line, t.col, t.token);
    }
}
