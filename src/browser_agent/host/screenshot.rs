//! Native deterministic PNG and PPM screenshot output.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::state::HostState;

#[path = "screenshot_capture.rs"]
mod capture;
#[cfg(test)]
#[path = "screenshot_tests.rs"]
mod tests;
#[path = "visual_compare.rs"]
pub(super) mod visual_compare;

pub(super) fn invoke(state: &HostState, payload: &Value) -> Result<Value, String> {
    let image = capture::image(state, payload)?;
    let (format, bytes) = encoded(
        &image,
        super::value::optional_string(payload, "path")?.as_deref(),
    )?;
    let Some(path) = super::value::optional_string(payload, "path")? else {
        return Ok(Value::Bytes(Rc::new(RefCell::new(bytes))));
    };
    std::fs::write(&path, bytes)
        .map_err(|error| format!("browser.screenshot: write `{}` failed: {}", path, error))?;
    Ok(super::value::map(vec![
        ("path", super::value::string(path)),
        ("format", super::value::string(format)),
        ("width", Value::Int(image.width as i64)),
        ("height", Value::Int(image.height as i64)),
    ]))
}

fn encoded(
    image: &crate::browser::RasterImage,
    path: Option<&str>,
) -> Result<(&'static str, Vec<u8>), String> {
    match path.map(|value| value.to_ascii_lowercase()) {
        Some(path) if path.ends_with(".ppm") => Ok(("ppm", image.to_ppm())),
        Some(path) if !path.ends_with(".png") => {
            Err("browser.screenshot: path must end in .png or .ppm".into())
        }
        _ => Ok(("png", super::png::encode(image))),
    }
}
