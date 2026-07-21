//! `tetherscript init` command.

use std::path::Path;

pub(crate) fn run(args: &[String]) {
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        print_help();
        return;
    }
    if args.len() > 1 {
        eprintln!("tetherscript init: expected at most one directory");
        std::process::exit(2);
    }
    let root = Path::new(args.first().map(String::as_str).unwrap_or("."));
    match crate::package::init(root) {
        Ok(manifest) => println!(
            "Created package `{}` at {}",
            manifest.name(),
            root.display()
        ),
        Err(error) => {
            eprintln!("tetherscript init: {error}");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("tetherscript init -- Create a local tetherscript package");
    println!("\nUSAGE:\n    tetherscript init [directory]");
}
