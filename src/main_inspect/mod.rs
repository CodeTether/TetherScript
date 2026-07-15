mod args;
mod frontend;
mod output;

use std::process;

pub(crate) fn run(arguments: &[String]) {
    let parsed = match args::parse(arguments) {
        Ok(value) => value,
        Err(error) => exit_usage(&error),
    };
    let args::Parsed::Run { mode, path } = parsed else {
        crate::main_inspect_help::print();
        return;
    };
    let source = crate::read_source(&path);
    let tokens = match frontend::tokens(&source) {
        Ok(value) => value,
        Err(error) => exit_error(&error),
    };
    match output::render(mode, tokens) {
        Ok(value) => println!("{value}"),
        Err(error) => exit_error(&error),
    }
}

fn exit_usage(error: &str) -> ! {
    eprintln!("tetherscript inspect: {error}");
    process::exit(2)
}

fn exit_error(error: &str) -> ! {
    eprintln!("tetherscript: {error}");
    process::exit(1)
}
