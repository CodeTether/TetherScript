//! Named logical cursors for deterministic window-relative automation.
//!
//! Cursor movement is local state only. OS pointer movement happens only when a
//! click or drag is dispatched through the existing computer bridge.

use std::collections::HashMap;

use crate::value::Value;

use super::authority::{ComputerAuthority, LogicalCursor};
use super::value::{owned_map, str_value};

impl ComputerAuthority {
    pub(crate) fn invoke_cursor(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        match method {
            "cursor_set" => self.cursor_set(args),
            "cursor_move" => self.cursor_move(args),
            "cursor_state" => self.cursor_state(args),
            "cursor_click" => self.cursor_click(args),
            "cursor_drag" => self.cursor_drag(args),
            "cursor_snapshot" => self.cursor_snapshot(args),
            _ => Err(format!("computer: no cursor method `{}`", method)),
        }
    }

    fn cursor_set(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_set", args)?;
        let name = get_str(&params, "name")?.to_string();
        let cursor = LogicalCursor {
            hwnd: get_i64(&params, "hwnd")?,
            x: get_f64(&params, "x")?,
            y: get_f64(&params, "y")?,
            client_area: get_bool_default(&params, "client_area", true)?,
        };
        self.cursors.borrow_mut().insert(name.clone(), cursor);
        Ok(cursor_value(&name, &cursor))
    }

    fn cursor_move(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_move", args)?;
        let name = get_str(&params, "name")?.to_string();
        let dx = get_f64(&params, "dx")?;
        let dy = get_f64(&params, "dy")?;
        let mut cursors = self.cursors.borrow_mut();
        let cursor = cursors
            .get_mut(&name)
            .ok_or_else(|| format!("computer.cursor_move: unknown cursor `{}`", name))?;
        cursor.x += dx;
        cursor.y += dy;
        Ok(cursor_value(&name, cursor))
    }

    fn cursor_state(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_state", args)?;
        let name = get_str(&params, "name")?;
        let cursors = self.cursors.borrow();
        let cursor = cursors
            .get(name)
            .ok_or_else(|| format!("computer.cursor_state: unknown cursor `{}`", name))?;
        Ok(cursor_value(name, cursor))
    }

    fn cursor_click(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_click", args)?;
        let name = get_str(&params, "name")?;
        let cursor = self.get_cursor(name, "cursor_click")?;
        let button = get_optional_str(&params, "button")?.unwrap_or("left");
        let action = match button {
            "left" => "click",
            "right" => "right_click",
            other => {
                return Err(format!(
                    "computer.cursor_click: button must be left/right, got `{}`",
                    other
                ))
            }
        };
        self.call_cursor_action(action, cursor_payload(&cursor, Vec::new()))
    }

    fn cursor_drag(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_drag", args)?;
        let name = get_str(&params, "name")?;
        let dx = get_f64(&params, "dx")?;
        let dy = get_f64(&params, "dy")?;
        let mut cursor = self.get_cursor(name, "cursor_drag")?;
        let x2 = cursor.x + dx;
        let y2 = cursor.y + dy;
        let mut extra = vec![
            ("x2".into(), number_value(x2)),
            ("y2".into(), number_value(y2)),
        ];
        for key in ["button", "duration_ms", "steps"] {
            if let Some(value) = params.get(key) {
                extra.push((key.into(), value.clone()));
            }
        }
        let result = self.call_cursor_action("drag", cursor_payload(&cursor, extra))?;
        cursor.x = x2;
        cursor.y = y2;
        self.cursors.borrow_mut().insert(name.to_string(), cursor);
        Ok(result)
    }

    fn cursor_snapshot(&self, args: &[Value]) -> Result<Value, String> {
        let params = one_map("cursor_snapshot", args)?;
        let name = get_str(&params, "name")?;
        let cursor = self.get_cursor(name, "cursor_snapshot")?;
        self.call_cursor_action(
            "window_snapshot",
            owned_map(vec![("hwnd".into(), Value::Int(cursor.hwnd))]),
        )
    }

    fn get_cursor(&self, name: &str, method: &str) -> Result<LogicalCursor, String> {
        self.cursors
            .borrow()
            .get(name)
            .copied()
            .ok_or_else(|| format!("computer.{}: unknown cursor `{}`", method, name))
    }
}

fn cursor_payload(cursor: &LogicalCursor, mut extra: Vec<(String, Value)>) -> Value {
    let mut entries = vec![
        ("hwnd".into(), Value::Int(cursor.hwnd)),
        ("x".into(), number_value(cursor.x)),
        ("y".into(), number_value(cursor.y)),
        ("client_area".into(), Value::Bool(cursor.client_area)),
    ];
    entries.append(&mut extra);
    owned_map(entries)
}

fn cursor_value(name: &str, cursor: &LogicalCursor) -> Value {
    owned_map(vec![
        ("name".into(), str_value(name)),
        ("hwnd".into(), Value::Int(cursor.hwnd)),
        ("x".into(), number_value(cursor.x)),
        ("y".into(), number_value(cursor.y)),
        ("client_area".into(), Value::Bool(cursor.client_area)),
    ])
}

fn number_value(value: f64) -> Value {
    if value.fract() == 0.0 {
        Value::Int(value as i64)
    } else {
        Value::Float(value)
    }
}

fn one_map(method: &str, args: &[Value]) -> Result<HashMap<String, Value>, String> {
    match args {
        [Value::Map(map)] => Ok(map.borrow().clone()),
        _ => Err(format!("computer.{} expects one params map", method)),
    }
}

fn get_str<'a>(map: &'a HashMap<String, Value>, key: &str) -> Result<&'a str, String> {
    match map.get(key) {
        Some(Value::Str(value)) => Ok(value.as_str()),
        Some(other) => Err(format!(
            "computer cursor `{}` must be str, got {}",
            key,
            other.type_name()
        )),
        None => Err(format!("computer cursor missing `{}`", key)),
    }
}

fn get_optional_str<'a>(
    map: &'a HashMap<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, String> {
    match map.get(key) {
        Some(Value::Str(value)) => Ok(Some(value.as_str())),
        Some(other) => Err(format!(
            "computer cursor `{}` must be str, got {}",
            key,
            other.type_name()
        )),
        None => Ok(None),
    }
}

fn get_i64(map: &HashMap<String, Value>, key: &str) -> Result<i64, String> {
    match map.get(key) {
        Some(Value::Int(value)) => Ok(*value),
        Some(other) => Err(format!(
            "computer cursor `{}` must be int, got {}",
            key,
            other.type_name()
        )),
        None => Err(format!("computer cursor missing `{}`", key)),
    }
}

fn get_f64(map: &HashMap<String, Value>, key: &str) -> Result<f64, String> {
    match map.get(key) {
        Some(Value::Int(value)) => Ok(*value as f64),
        Some(Value::Float(value)) => Ok(*value),
        Some(other) => Err(format!(
            "computer cursor `{}` must be number, got {}",
            key,
            other.type_name()
        )),
        None => Err(format!("computer cursor missing `{}`", key)),
    }
}

fn get_bool_default(
    map: &HashMap<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, String> {
    match map.get(key) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(other) => Err(format!(
            "computer cursor `{}` must be bool, got {}",
            key,
            other.type_name()
        )),
        None => Ok(default),
    }
}
