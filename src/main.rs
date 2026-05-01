//! tetherscript — a dynamically-typed scripting language with Rust-style ownership.
//!
//! Usage:
//!   tetherscript <file.tether>                  # run (tree-walking interpreter)
//!   tetherscript --vm       <file.tether>       # run (bytecode VM)
//!   tetherscript --tokens   <file.tether>       # dump tokens and exit
//!   tetherscript --ast      <file.tether>       # dump AST and exit
//!   tetherscript --bytecode <file.tether>       # dump compiled bytecode and exit
//!   tetherscript --lsp                      # serve LSP over stdio

mod ast;
mod bytecode;
mod capability;
mod compiler;
mod fs_cap;
mod http;
mod interp;
mod json;
mod lexer;
mod lsp;
mod output;
mod parser;
mod provider_cap;
mod rpc_cap;
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
        eprintln!("usage: tetherscript [--tokens|--ast|--bytecode|--vm|--lsp] [--step-budget <n>] [--grant-fs <root>] [--grant-provider <endpoint>] [--grant-rpc <endpoint>] <file.tether>");
        process::exit(2);
    }

    if args[1] == "--lsp" {
        if let Err(e) = lsp::run() {
            eprintln!("tetherscript-lsp: {}", e);
            process::exit(1);
        }
        return;
    }

    let mut mode = "run";
    let mut path: Option<String> = None;
    let mut step_budget: Option<u64> = None;
    let mut fs_grant: Option<String> = None;
    let mut provider_grant: Option<String> = None;
    let mut rpc_grant: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tokens" => {
                mode = "tokens";
                i += 1;
            }
            "--ast" => {
                mode = "ast";
                i += 1;
            }
            "--bytecode" => {
                mode = "bytecode";
                i += 1;
            }
            "--vm" => {
                mode = "vm";
                i += 1;
            }
            "--step-budget" => {
                if i + 1 >= args.len() {
                    eprintln!("tetherscript: --step-budget requires an integer argument");
                    process::exit(2);
                }
                match args[i + 1].parse::<u64>() {
                    Ok(n) => step_budget = Some(n),
                    Err(_) => {
                        eprintln!("tetherscript: --step-budget must be a non-negative integer");
                        process::exit(2);
                    }
                }
                i += 2;
            }
            "--grant-fs" => {
                if i + 1 >= args.len() {
                    eprintln!("tetherscript: --grant-fs requires a directory argument");
                    process::exit(2);
                }
                fs_grant = Some(args[i + 1].clone());
                i += 2;
            }
            "--grant-provider" => {
                if i + 1 >= args.len() {
                    eprintln!("tetherscript: --grant-provider requires an http:// endpoint argument");
                    process::exit(2);
                }
                let endpoint = &args[i + 1];
                if !endpoint.starts_with("http://") {
                    eprintln!("tetherscript: --grant-provider endpoint must start with http://");
                    process::exit(2);
                }
                provider_grant = Some(endpoint.clone());
                i += 2;
            }
            "--grant-rpc" => {
                if i + 1 >= args.len() {
                    eprintln!("tetherscript: --grant-rpc requires an http:// endpoint argument");
                    process::exit(2);
                }
                let endpoint = &args[i + 1];
                if !endpoint.starts_with("http://") {
                    eprintln!("tetherscript: --grant-rpc endpoint must start with http://");
                    process::exit(2);
                }
                rpc_grant = Some(endpoint.clone());
                i += 2;
            }
            other => {
                if path.is_some() {
                    eprintln!("tetherscript: unexpected argument `{}`", other);
                    process::exit(2);
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let path = match path {
        Some(path) => path,
        None => {
            eprintln!("tetherscript: missing source file");
            process::exit(2);
        }
    };

    let src = match fs::read_to_string(&path) {
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
        if let Some(root) = &fs_grant {
            vm.grant("fs", fs_cap::FsAuthority::new(root));
        }
        if let Some(endpoint) = &provider_grant {
            vm.grant("provider", provider_cap::ProviderAuthority::new(endpoint));
        }
        if let Some(endpoint) = &rpc_grant {
            vm.grant("rpc", rpc_cap::RpcAuthority::new(endpoint));
        }
        let result = if let Some(budget) = step_budget {
            interp::with_step_budget(budget, || vm.run(chunk))
        } else {
            vm.run(chunk)
        };
        if let Err(e) = result {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
        }
        return;
    }

    let mut interp = Interpreter::new();
    if let Some(root) = &fs_grant {
        interp.grant("fs", fs_cap::FsAuthority::new(root));
    }
    if let Some(endpoint) = &provider_grant {
        interp.grant("provider", provider_cap::ProviderAuthority::new(endpoint));
    }
    if let Some(endpoint) = &rpc_grant {
        interp.grant("rpc", rpc_cap::RpcAuthority::new(endpoint));
    }
    let result = if let Some(budget) = step_budget {
        interp::with_step_budget(budget, || interp.run(&program))
    } else {
        interp.run(&program)
    };
    if let Err(e) = result {
        eprintln!("tetherscript: {}", e);
        process::exit(1);
    }
}
