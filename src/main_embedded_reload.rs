//! Hot-reload loop for embedded launchers with optional sidecar source.

use std::fs;

#[path = "main_embedded_rollback.rs"]
mod rollback;

pub(crate) fn run(embedded: &str, args: &[String]) -> Result<(), String> {
    let opts = super::args::parse(args)?;
    let source_path = opts.reload_source.clone();
    let mut previous = source(embedded, source_path.as_deref());
    let mut last_good = None;
    let mut recovered = false;
    loop {
        let src = source(embedded, source_path.as_deref());
        let hot = crate::main_caps::script_hot_reload(&src);
        if hot {
            super::marker::clear(source_path.as_ref());
        }
        if let Err(error) = super::run_source(&src, &opts) {
            let Some(good) = last_good.as_deref() else {
                return Err(error);
            };
            rollback::restore(source_path.as_deref(), good, &src, &error)?;
            if recovered {
                return Err(format!(
                    "reloaded generation failed after rollback: {error}"
                ));
            }
            previous = good.to_string();
            recovered = true;
            continue;
        }
        last_good = Some(src.clone());
        if !hot || !super::marker::take(source_path.as_ref()) {
            break;
        }
        let current = source(embedded, source_path.as_deref());
        if current == previous {
            break;
        }
        previous = current;
    }
    Ok(())
}

fn source(embedded: &str, path: Option<&str>) -> String {
    path.and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_else(|| embedded.to_string())
}
