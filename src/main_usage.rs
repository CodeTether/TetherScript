//! Short CLI usage text.

pub(crate) fn print_usage() {
    eprintln!("Usage: tetherscript <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  run [target]         Run a source file or local package");
    eprintln!("  build <file>         Build a standalone launcher");
    eprintln!("  check [target]       Analyze a source file or local package");
    eprintln!("  init [directory]     Create a local package");
    eprintln!("  inspect <file>       Inspect source (tokens, AST, IR, bytecode)");
    eprintln!("  render <html>        Render HTML/CSS display list");
    eprintln!("  raster <html> <ppm>  Render HTML/CSS to a PPM image");
    eprintln!("  js <file.js>         Run JavaScript with the built-in engine");
    eprintln!("  git                  Show first-class git workspace status");
    eprintln!("  repl                 Interactive REPL");
    eprintln!("  lsp                  Start LSP server over stdio");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help           Show help");
    eprintln!("  -V, --version        Show version");
    eprintln!();
    eprintln!("Run 'tetherscript <command> --help' for more on a command.");
    eprintln!();
    eprintln!("Legacy: tetherscript <file.tether> also works (same as 'run').");
}
