//! Blocking line input surfaced as a TUI event.

use std::io::{self, Write};

use crate::value::Value;

use super::val;

pub(super) fn read_event(args: &[Value]) -> Result<Value, String> {
    if args.len() > 1 {
        return Err("tui_read_event expects optional prompt".into());
    }
    let prompt = match args.first() {
        Some(Value::Str(text)) => text.as_str(),
        Some(other) => {
            return Err(format!(
                "tui_read_event: prompt must be str, got {}",
                other.type_name()
            ))
        }
        None => "",
    };
    Ok(val::result(read_line(prompt).map(|text| {
        val::map_value([
            ("type".into(), val::strv("line")),
            ("text".into(), val::strv(text)),
        ])
    })))
}

fn read_line(prompt: &str) -> Result<String, String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| format!("tui_read_event: flush failed: {error}"))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("tui_read_event: read failed: {error}"))?;
    Ok(line
        .trim_end_matches(['\r', '\n'])
        .trim_start_matches('\u{feff}')
        .to_string())
}
