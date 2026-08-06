//! Package scaffolding commands.

use std::path::Path;

pub(crate) fn run(command: &str, args: &[String]) {
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        print_help(command);
        return;
    }

    if let Err(error) = validate_args(command, args) {
        eprintln!("tetherscript {command}: {error}");
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
            eprintln!("tetherscript {command}: {error}");
            std::process::exit(1);
        }
    }
}

fn validate_args(command: &str, args: &[String]) -> Result<(), String> {
    if command == "new" && args.is_empty() {
        return Err("expected a project directory".into());
    }
    if args.len() > 1 {
        return Err("expected at most one directory".into());
    }
    Ok(())
}

fn print_help(command: &str) {
    match command {
        "new" => {
            println!("tetherscript new -- Scaffold a new tetherscript package");
            println!("\nUSAGE:\n    tetherscript new <directory>");
        }
        _ => {
            println!("tetherscript init -- Create a local tetherscript package");
            println!("\nUSAGE:\n    tetherscript init [directory]");
        }
    }
}
