//! Deterministic comparison of native-host screenshot artifacts.

use crate::value::Value;

use super::super::state::HostState;

#[path = "visual_compare_bytes.rs"]
mod bytes;
#[cfg(test)]
#[path = "visual_compare_tests.rs"]
mod tests;

pub(in super::super) fn invoke(state: &HostState, payload: &Value) -> Result<Value, String> {
    let mode = super::super::value::string_field(payload, "mode")?;
    let before_path = super::super::value::string_field(payload, "before")?;
    let before = read(&before_path)?;
    let after = if mode == "assert_screenshot_matches" {
        super::super::png::encode(&state.page.screenshot()?)
    } else {
        read(&super::super::value::string_field(payload, "after")?)?
    };
    let changed = bytes::changed(&before, &after);
    if mode == "assert_screenshot_matches" && changed != 0 {
        return Err(format!(
            "browser.assert_screenshot_matches: `{before_path}` differs by {changed} encoded bytes"
        ));
    }
    Ok(super::super::value::map(vec![
        ("matches", Value::Bool(changed == 0)),
        ("changed_bytes", Value::Int(changed as i64)),
        ("before_bytes", Value::Int(before.len() as i64)),
        ("after_bytes", Value::Int(after.len() as i64)),
    ]))
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path)
        .map_err(|error| format!("browser.visual_compare: read `{path}` failed: {error}"))
}
