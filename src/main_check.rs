//! Package-aware `tetherscript check` command.

use std::path::Path;

pub(crate) fn run(args: &[String]) {
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        print_help();
        return;
    }
    if args.len() > 1 {
        eprintln!("tetherscript check: expected at most one file or package directory");
        std::process::exit(2);
    }
    let path = crate::main_target::resolve(args.first().map(String::as_str), "check");
    let program = crate::modules::load_program(Path::new(&path)).unwrap_or_else(|error| {
        eprintln!("tetherscript check: {error}");
        std::process::exit(1);
    });
    match crate::ownership::analyze(&program) {
        Ok(()) => println!("{path}: ok"),
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!(
                    "tetherscript check: ownership error: {}",
                    diagnostic.message
                );
            }
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("tetherscript check -- Parse and analyze a file or local package");
    println!("\nUSAGE:\n    tetherscript check [file.tether|package-directory]");
}
