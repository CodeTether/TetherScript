//! Standalone native browser action host.

fn main() {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:41707".into());
    if let Err(error) = tetherscript::browser_agent::host::serve(&address) {
        eprintln!("tetherscript-browser-host: {}", error);
        std::process::exit(1);
    }
}
