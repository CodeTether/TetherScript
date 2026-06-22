//! ANSI escape helpers.

use crate::value::Value;

use super::val;

pub(super) fn clear(_: &[Value]) -> Result<Value, String> {
    Ok(val::strv("\x1b[2J\x1b[H"))
}

pub(super) fn cursor(args: &[Value]) -> Result<Value, String> {
    if val::bool_arg(&args[0], "tui_cursor: visible")? {
        Ok(val::strv("\x1b[?25h"))
    } else {
        Ok(val::strv("\x1b[?25l"))
    }
}

pub(super) fn alt_screen(args: &[Value]) -> Result<Value, String> {
    if val::bool_arg(&args[0], "tui_alt_screen: enabled")? {
        Ok(val::strv("\x1b[?1049h"))
    } else {
        Ok(val::strv("\x1b[?1049l"))
    }
}

pub(super) fn move_to(args: &[Value]) -> Result<Value, String> {
    let row = bounded(&args[0], "tui_move_to: row")?;
    let col = bounded(&args[1], "tui_move_to: col")?;
    Ok(val::strv(format!("\x1b[{row};{col}H")))
}

fn bounded(value: &Value, label: &str) -> Result<i64, String> {
    let value = val::int_arg(value, label)?;
    if value <= 0 {
        return Err(format!("{label} must be positive"));
    }
    Ok(value.min(10_000))
}
