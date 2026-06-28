//! `tetherscript build` command.

use std::path::Path;

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let opts = crate::main_build_parse::parse(args)?;
    let source = std::fs::read_to_string(&opts.path)
        .map_err(|e| format!("can't read {}: {e}", opts.path))?;
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
