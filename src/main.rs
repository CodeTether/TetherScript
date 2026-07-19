//! tetherscript — a dynamically-typed scripting language with Rust-style ownership.
//!
//! A standalone CLI tool for running TetherScript programs.
//!
//! # Usage
//!
//! ```text
//! tetherscript run <file.tether>              run with bytecode VM (default)
//! tetherscript run --interp <file.tether>     run with tree-walking interpreter
//! tetherscript check <file>                   parse and run ownership analysis
//! tetherscript render <html-file> [css-file] [width] render HTML/CSS files to a text display list
//! tetherscript raster <html-file> <output.ppm> [css-file] [width] [height] [scale] render to pixels
//! tetherscript inspect --tokens <file>        dump tokens
//! tetherscript inspect --ast <file>           dump AST
//! tetherscript inspect --ir <file>            dump Tether IR
//! tetherscript inspect --bytecode <file>      dump compiled bytecode
//! tetherscript inspect --bytecode-visual <file> render annotated bytecode
//! tetherscript lsp                            serve LSP over stdio
//! tetherscript repl                           interactive REPL
//! tetherscript --help                         show help
//! tetherscript --version                      show version
//! ```

#![allow(dead_code, clippy::too_many_arguments)]

mod ast;
mod browser;
mod browser_cap;
mod browser_cookie;
mod browser_js;
mod bytecode;
mod bytecode_visual;
mod capability;
mod cli_args;
mod compiler;
mod embed;
mod embed_perm;
mod fs_cap;
mod git_tui;
mod http;
mod interp;
mod ir;
mod js;
mod json;
mod lexer;
mod lsp;
mod main_build;
mod main_build_parse;
mod main_caps;
mod main_embedded;
mod main_help;
mod main_help_examples;
mod main_inspect;
mod main_inspect_help;
mod main_run_help;
mod main_usage;
mod output;
mod ownership;
mod parser;
mod process_control;
mod provider_cap;
mod provider_env;
mod provider_vault;
mod rpc_cap;
mod run_reload;
mod scheduler;
mod smtp;
mod system;
mod template;
mod tls;
mod token;
mod value;
mod vm;
mod zlib;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use lexer::Lexer;
use main_caps::RunCaps;
use main_usage::print_usage;
use parser::Parser;
use vm::VM;
use {compiler::Compiler, interp::Interpreter};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(src) = embed::read_current() {
        let script_args: Vec<String> = args.iter().skip(1).cloned().collect();
        if let Err(e) = main_embedded::run(&src, &script_args) {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
        }
        return;
    }

    // No arguments at all
    if args.len() < 2 {
        print_usage();
        process::exit(2);
    }

    let first = &args[1];

    // Global flags (before subcommand)
    match first.as_str() {
        "--help" | "-h" => {
            main_help::print();
            return;
        }
        "--version" | "-V" | "-v" => {
            println!("tetherscript {}", VERSION);
            return;
        }
        _ => {}
    }

    // Subcommands
    match first.as_str() {
        "run" => cmd_run(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "render" => cmd_render(&args[2..]),
        "raster" => cmd_raster(&args[2..]),
        "js" => cmd_js(&args[2..]),
        "inspect" => main_inspect::run(&args[2..]),
        "lsp" => cmd_lsp(),
        "repl" => cmd_repl(),
        "git" => cmd_git(),
        // Legacy: bare file path as first arg (backward compat)
        other => {
            // If it looks like a flag, error
            if other.starts_with('-') {
                eprintln!("tetherscript: unknown option '{}'", other);
                eprintln!("Try 'tetherscript --help' for usage.");
                process::exit(2);
            }
            // Treat as: tetherscript <file> (legacy run mode)
            cmd_run_legacy(&args[1..]);
        }
    }
}

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

fn cmd_build(args: &[String]) {
    match main_build::run(args) {
        Ok(()) => {}
        Err(e) if e == "__help__" => main_build::print_help(),
        Err(e) => {
            eprintln!("tetherscript build: {}", e);
            process::exit(2);
        }
    }
}

fn cmd_run(args: &[String]) {
    let mut vm_mode = true;
    let mut step_budget: Option<u64> = None;
    let mut fs_grant: Option<String> = None;
    let mut full_access = false;
    let mut provider_grant: Option<String> = None;
    let mut provider_key: Option<String> = None;
    let mut provider_vault: Option<String> = None;
    let mut rpc_grant: Option<String> = None;
    let mut browser_grant: Option<String> = None;
    let mut browser_origins: Vec<String> = Vec::new();
    let mut browser_scopes: Vec<String> = Vec::new();
    let mut script_args: Vec<String> = Vec::new();
    let mut path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                main_run_help::print();
                return;
            }
            "--vm" => {
                vm_mode = true;
                i += 1;
            }
            "--interp" | "--tree-walk" => {
                vm_mode = false;
                i += 1;
            }
            "--step-budget" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --step-budget requires an integer argument");
                    process::exit(2);
                }
                match args[i].parse::<u64>() {
                    Ok(n) => step_budget = Some(n),
                    Err(_) => {
                        eprintln!("tetherscript run: --step-budget must be a non-negative integer");
                        process::exit(2);
                    }
                }
                i += 1;
            }
            "--grant-fs" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --grant-fs requires a directory argument");
                    process::exit(2);
                }
                fs_grant = Some(args[i].clone());
                i += 1;
            }
            "--access-mode" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --access-mode requires restricted or full");
                    process::exit(2);
                }
                full_access = parse_access_mode(&args[i], "tetherscript run");
                i += 1;
            }
            "--grant-provider" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --grant-provider requires an http(s):// endpoint");
                    process::exit(2);
                }
                if !args[i].starts_with("http://") && !args[i].starts_with("https://") {
                    eprintln!("tetherscript run: --grant-provider endpoint must start with http:// or https://");
                    process::exit(2);
                }
                provider_grant = Some(args[i].clone());
                i += 1;
            }
            "--grant-provider-key" => {
                i += 1;
                if i >= args.len() {
                    eprintln!(
                        "tetherscript run: --grant-provider-key requires an API key argument"
                    );
                    process::exit(2);
                }
                provider_key = Some(args[i].clone());
                i += 1;
            }
            "--grant-provider-vault" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --grant-provider-vault requires a provider id");
                    process::exit(2);
                }
                provider_vault = Some(args[i].clone());
                i += 1;
            }
            "--grant-rpc" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --grant-rpc requires an http:// endpoint");
                    process::exit(2);
                }
                if !args[i].starts_with("http://") {
                    eprintln!("tetherscript run: --grant-rpc endpoint must start with http://");
                    process::exit(2);
                }
                rpc_grant = Some(args[i].clone());
                i += 1;
            }
            "--grant-browser" => {
                i += 1;
                if i >= args.len() {
                    eprintln!(
                        "tetherscript run: --grant-browser requires a browser bridge endpoint"
                    );
                    process::exit(2);
                }
                if !args[i].starts_with("http://") && !args[i].starts_with("https://") {
                    eprintln!("tetherscript run: --grant-browser endpoint must start with http:// or https://");
                    process::exit(2);
                }
                browser_grant = Some(args[i].clone());
                i += 1;
            }
            "--browser-origin" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --browser-origin requires an origin");
                    process::exit(2);
                }
                browser_origins.push(args[i].clone());
                i += 1;
            }
            "--browser-scope" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript run: --browser-scope requires a scope name or 'all'");
                    process::exit(2);
                }
                if args[i] == "all" {
                    browser_scopes.extend(browser_cap::BrowserAuthority::all_scopes());
                } else {
                    browser_scopes.push(args[i].clone());
                }
                i += 1;
            }
            other => {
                if other == "--" {
                    i += 1;
                    script_args.extend(args[i..].iter().cloned());
                    break;
                }
                if path.is_some() {
                    script_args.push(other.to_string());
                    i += 1;
                    continue;
                }
                if other.starts_with('-') {
                    eprintln!("tetherscript run: unknown option '{}'", other);
                    process::exit(2);
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("tetherscript run: missing source file");
            eprintln!("Try 'tetherscript run --help' for usage.");
            process::exit(2);
        }
    };

    run_reload::execute(
        &path,
        vm_mode,
        step_budget,
        &fs_grant,
        full_access,
        &provider_grant,
        &provider_key,
        &provider_vault,
        &rpc_grant,
        &browser_grant,
        &browser_origins,
        &browser_scopes,
        &script_args,
    );
}

fn cmd_lsp() {
    if let Err(e) = lsp::run() {
        eprintln!("tetherscript-lsp: {}", e);
        process::exit(1);
    }
}

fn cmd_render(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!("tetherscript render -- Render HTML/CSS to a deterministic text display list");
        println!();
        println!("USAGE:");
        println!("    tetherscript render <html-file> [css-file] [width]");
        if args.is_empty() {
            process::exit(2);
        }
        return;
    }
    let html = read_source(&args[0]);
    let css = args
        .get(1)
        .map(|path| read_source(path))
        .unwrap_or_default();
    let width = args
        .get(2)
        .map(|raw| {
            raw.parse::<i64>().unwrap_or_else(|_| {
                eprintln!("tetherscript render: width must be an integer");
                process::exit(2);
            })
        })
        .unwrap_or(80);
    let doc = browser::parse_html(&html);
    let layout = browser::layout_document(&doc, &css, width);
    print!("{}", browser::render_text(&layout));
}

fn cmd_raster(args: &[String]) {
    if args.len() < 2 || args[0] == "--help" || args[0] == "-h" {
        println!(
            "tetherscript raster -- Render HTML/CSS to a native software-rasterized PPM image"
        );
        println!();
        println!("USAGE:");
        println!(
            "    tetherscript raster <html-file> <output.ppm> [css-file] [width] [height] [scale]"
        );
        if args.len() < 2 {
            process::exit(2);
        }
        return;
    }
    let html = read_source(&args[0]);
    let output = &args[1];
    let css = args
        .get(2)
        .map(|path| read_source(path))
        .unwrap_or_default();
    let width = parse_i64_arg(args.get(3), "tetherscript raster: width", 80);
    let height = args
        .get(4)
        .map(|raw| parse_i64_arg(Some(raw), "tetherscript raster: height", 0))
        .filter(|height| *height > 0);
    let scale = parse_i64_arg(args.get(5), "tetherscript raster: scale", 8);
    if scale <= 0 {
        eprintln!("tetherscript raster: scale must be positive");
        process::exit(2);
    }
    let doc = browser::parse_html(&html);
    let image = browser::render_document_to_raster(
        &doc,
        &css,
        browser::RenderOptions {
            viewport_width: width,
            viewport_height: height,
            scale: scale as usize,
            ..browser::RenderOptions::default()
        },
    )
    .unwrap_or_else(|err| {
        eprintln!("tetherscript raster: {}", err);
        process::exit(1);
    });
    fs::write(output, image.to_ppm()).unwrap_or_else(|err| {
        eprintln!("tetherscript raster: failed to write '{}': {}", output, err);
        process::exit(1);
    });
}

fn cmd_js(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!(
            "tetherscript js -- Run a JavaScript file with the built-in no-dependency JS engine"
        );
        println!();
        println!("USAGE:");
        println!("    tetherscript js <file.js>");
        if args.is_empty() {
            process::exit(2);
        }
        return;
    }
    let src = read_source(&args[0]);
    let mut engine = js::JsEngine::new();
    match engine.eval(&src) {
        Ok(value) => {
            for line in engine.console_output() {
                println!("{}", line);
            }
            if !matches!(value, js::JsValue::Undefined) {
                println!("{}", value.display());
            }
        }
        Err(e) => {
            eprintln!("tetherscript js: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_git() {
    match git_tui::load_panel(std::path::Path::new(".")) {
        Ok(panel) => print!("{}", git_tui::render_panel(&panel)),
        Err(error) => {
            eprintln!("tetherscript git: {error}");
            process::exit(1);
        }
    }
}

fn cmd_check(args: &[String]) {
    if args.len() != 1 || args[0] == "--help" || args[0] == "-h" {
        println!("tetherscript check -- Parse source and run static ownership analysis");
        println!();
        println!("USAGE:");
        println!("    tetherscript check <file.tether>");
        if args.len() != 1 {
            process::exit(2);
        }
        return;
    }

    let src = read_source(&args[0]);
    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "tetherscript check: lex error at {}:{}: {}",
                e.line, e.col, e.msg
            );
            process::exit(1);
        }
    };
    let program = match Parser::new(tokens).parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "tetherscript check: parse error at {}:{}: {}",
                e.line, e.col, e.msg
            );
            process::exit(1);
        }
    };
    match ownership::analyze(&program) {
        Ok(()) => println!("{}: ok", args[0]),
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!(
                    "tetherscript check: ownership error: {}",
                    diagnostic.message
                );
            }
            process::exit(1);
        }
    }
}

fn cmd_repl() {
    println!("TetherScript {} REPL", VERSION);
    println!("Type expressions or statements. Ctrl+C to exit.");
    println!();

    let mut interp = Interpreter::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed == "help" {
            println!("  Enter TetherScript expressions or statements.");
            println!("  'exit' or 'quit' to leave.");
            continue;
        }

        let tokens = match Lexer::new(&line).tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  lex error: {}:{}: {}", e.line, e.col, e.msg);
                continue;
            }
        };

        let program = match Parser::new(tokens).parse_program() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  parse error: {}:{}: {}", e.line, e.col, e.msg);
                continue;
            }
        };

        match interp.run_repl(&program) {
            Ok(value) => println!("  {}", value),
            Err(e) => eprintln!("  error: {}", e),
        }
    }
}

/// Legacy mode: bare `tetherscript <file>` without subcommand.
fn cmd_run_legacy(args: &[String]) {
    let mut vm_mode = true;
    let mut step_budget: Option<u64> = None;
    let mut fs_grant: Option<String> = None;
    let mut full_access = false;
    let mut provider_grant: Option<String> = None;
    let mut provider_key: Option<String> = None;
    let mut provider_vault: Option<String> = None;
    let mut rpc_grant: Option<String> = None;
    let mut browser_grant: Option<String> = None;
    let mut browser_origins: Vec<String> = Vec::new();
    let mut browser_scopes: Vec<String> = Vec::new();
    let mut path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--vm" => {
                vm_mode = true;
                i += 1;
            }
            "--interp" | "--tree-walk" => {
                vm_mode = false;
                i += 1;
            }
            "--step-budget" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --step-budget requires an integer argument");
                    process::exit(2);
                }
                match args[i].parse::<u64>() {
                    Ok(n) => step_budget = Some(n),
                    Err(_) => {
                        eprintln!("tetherscript: --step-budget must be a non-negative integer");
                        process::exit(2);
                    }
                }
                i += 1;
            }
            "--grant-fs" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-fs requires a directory argument");
                    process::exit(2);
                }
                fs_grant = Some(args[i].clone());
                i += 1;
            }
            "--access-mode" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --access-mode requires restricted or full");
                    process::exit(2);
                }
                full_access = parse_access_mode(&args[i], "tetherscript");
                i += 1;
            }
            "--grant-provider" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-provider requires an http(s):// endpoint");
                    process::exit(2);
                }
                if !args[i].starts_with("http://") && !args[i].starts_with("https://") {
                    eprintln!("tetherscript: --grant-provider endpoint must start with http:// or https://");
                    process::exit(2);
                }
                provider_grant = Some(args[i].clone());
                i += 1;
            }
            "--grant-provider-key" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-provider-key requires an API key argument");
                    process::exit(2);
                }
                provider_key = Some(args[i].clone());
                i += 1;
            }
            "--grant-provider-vault" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-provider-vault requires a provider id");
                    process::exit(2);
                }
                provider_vault = Some(args[i].clone());
                i += 1;
            }
            "--grant-rpc" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-rpc requires an http:// endpoint");
                    process::exit(2);
                }
                if !args[i].starts_with("http://") {
                    eprintln!("tetherscript: --grant-rpc endpoint must start with http://");
                    process::exit(2);
                }
                rpc_grant = Some(args[i].clone());
                i += 1;
            }
            "--grant-browser" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --grant-browser requires a browser bridge endpoint");
                    process::exit(2);
                }
                browser_grant = Some(args[i].clone());
                i += 1;
            }
            "--browser-origin" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --browser-origin requires an origin");
                    process::exit(2);
                }
                browser_origins.push(args[i].clone());
                i += 1;
            }
            "--browser-scope" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("tetherscript: --browser-scope requires a scope name or 'all'");
                    process::exit(2);
                }
                if args[i] == "all" {
                    browser_scopes.extend(browser_cap::BrowserAuthority::all_scopes());
                } else {
                    browser_scopes.push(args[i].clone());
                }
                i += 1;
            }
            "--help" | "-h" => {
                main_help::print();
                return;
            }
            "--version" | "-V" => {
                println!("tetherscript {}", VERSION);
                return;
            }
            other => {
                if other.starts_with('-') {
                    eprintln!("tetherscript: unknown option '{}'", other);
                    eprintln!("Try 'tetherscript --help' for usage.");
                    process::exit(2);
                }
                if path.is_some() {
                    eprintln!("tetherscript: unexpected argument '{}'", other);
                    process::exit(2);
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("tetherscript: missing source file");
            eprintln!("Try 'tetherscript --help' for usage.");
            process::exit(2);
        }
    };

    run_reload::execute(
        &path,
        vm_mode,
        step_budget,
        &fs_grant,
        full_access,
        &provider_grant,
        &provider_key,
        &provider_vault,
        &rpc_grant,
        &browser_grant,
        &browser_origins,
        &browser_scopes,
        &[],
    );
}

// ---------------------------------------------------------------------------
// Shared execution
// ---------------------------------------------------------------------------

fn read_source(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tetherscript: can't read {}: {}", path, e);
            process::exit(1);
        }
    }
}

fn parse_i64_arg(value: Option<&String>, label: &str, default: i64) -> i64 {
    value
        .map(|raw| {
            raw.parse::<i64>().unwrap_or_else(|_| {
                eprintln!("{} must be an integer", label);
                process::exit(2);
            })
        })
        .unwrap_or(default)
}

fn execute_file(
    path: &str,
    vm_mode: bool,
    step_budget: Option<u64>,
    fs_grant: &Option<String>,
    full_access: bool,
    provider_grant: &Option<String>,
    provider_key: &Option<String>,
    provider_vault: &Option<String>,
    rpc_grant: &Option<String>,
    browser_grant: &Option<String>,
    browser_origins: &[String],
    browser_scopes: &[String],
    script_args: &[String],
) {
    let src = read_source(path);
    let full_access = main_caps::script_full_access(&src, full_access);

    let tokens = match Lexer::new(&src).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tetherscript: lex error at {}:{}: {}", e.line, e.col, e.msg);
            process::exit(1);
        }
    };

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

    if let Err(diagnostics) = ownership::analyze(&program) {
        for diagnostic in diagnostics {
            eprintln!("tetherscript: ownership error: {}", diagnostic.message);
        }
        process::exit(1);
    }

    if vm_mode {
        let chunk = Compiler::compile_program(&program);
        let mut vm = VM::new();
        vm.set_instruction_budget(step_budget);
        vm.install_cli_args(script_args);
        let caps = RunCaps {
            fs_grant,
            full_access,
            provider_grant,
            provider_key,
            provider_vault,
            rpc_grant,
            browser_grant,
            browser_origins,
            browser_scopes,
        };
        if let Err(e) = main_caps::grant_vm(&mut vm, &caps) {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
        }
        let result = vm.run(chunk);
        if let Err(e) = result {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
        }
    } else {
        let mut interp = Interpreter::new();
        interp.install_cli_args(script_args);
        let caps = RunCaps {
            fs_grant,
            full_access,
            provider_grant,
            provider_key,
            provider_vault,
            rpc_grant,
            browser_grant,
            browser_origins,
            browser_scopes,
        };
        if let Err(e) = main_caps::grant_interp(&mut interp, &caps) {
            eprintln!("tetherscript: {}", e);
            process::exit(1);
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
}

fn parse_access_mode(value: &str, label: &str) -> bool {
    match value {
        "restricted" => false,
        "full" => true,
        _ => {
            eprintln!("{label}: --access-mode must be restricted or full");
            process::exit(2);
        }
    }
}
