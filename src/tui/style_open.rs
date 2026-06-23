//! ANSI open sequence serialization for parsed TUI styles.

use super::{style::Style, style_color};

pub(super) fn open(style: &Style) -> Result<String, String> {
    let mut codes = Vec::new();
    push_bool(&mut codes, style.bold, "1");
    push_bool(&mut codes, style.dim, "2");
    push_bool(&mut codes, style.underline, "4");
    push_bool(&mut codes, style.inverse, "7");
    push_color(&mut codes, "fg", &style.fg, style_color::fg_code)?;
    push_color(&mut codes, "bg", &style.bg, style_color::bg_code)?;
    if codes.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("\x1b[{}m", codes.join(";")))
    }
}

fn push_bool(codes: &mut Vec<String>, enabled: bool, code: &str) {
    if enabled {
        codes.push(code.to_string());
    }
}

fn push_color(
    codes: &mut Vec<String>,
    label: &str,
    value: &Option<String>,
    lookup: fn(&str) -> Option<&'static str>,
) -> Result<(), String> {
    if let Some(name) = value {
        let code = lookup(name).ok_or_else(|| format!("tui_style: unknown {label} {name}"))?;
        codes.push(code.to_string());
    }
    Ok(())
}
