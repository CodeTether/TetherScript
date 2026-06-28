//! Long-form top-level CLI help.

use crate::VERSION;

pub(crate) fn print() {
    println!(
        "TetherScript {} -- a scripting language with Rust-style ownership",
        VERSION
    );
    println!();
    println!("USAGE:");
    println!("    tetherscript <command> [options]");
    println!("    tetherscript <file.tether> [options]    (legacy, same as 'run')");
    println!();
    println!("COMMANDS:");
    for line in COMMANDS {
        println!("    {line}");
    }
    println!();
    println!("CAPABILITIES:");
    println!("    TetherScript uses capability-based security. Scripts cannot access");
    println!("    the filesystem, network, or LLM APIs unless explicitly granted.");
    println!();
    println!("EXAMPLES:");
    for line in crate::main_help_examples::EXAMPLES {
        println!("    {line}");
    }
    println!();
    println!("MORE INFO:");
    println!("    https://github.com/CodeTether/TetherScript");
}

const COMMANDS: &[&str] = &[
    "run <file>        Run a TetherScript program",
    "build <file>      Build a standalone executable launcher",
    "inspect <file>    Inspect frontend output (tokens, AST, bytecode)",
    "render <html>     Render HTML/CSS to a display list",
    "raster <html>     Render HTML/CSS to a native PPM image",
    "js <file.js>      Run JavaScript with the built-in engine",
    "git               Show first-class git workspace status",
    "repl              Start interactive REPL",
    "lsp               Start language server over stdio",
];
