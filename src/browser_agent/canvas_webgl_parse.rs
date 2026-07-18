//! WebGL rendering-state attribute parser.

use crate::browser::Element;

use super::canvas_webgl_attrs as attr;
use super::canvas_webgl_commands::parse_command;
use super::canvas_webgl_model::{WebGlCommand, WebGlContextSnapshot};

pub(super) fn snapshot_from_element(element: &Element) -> Option<WebGlContextSnapshot> {
    Some(WebGlContextSnapshot {
        version: attr::u8(element, "data-agent-webgl-version")?,
        width: attr::u32(element, "data-agent-webgl-width")
            .or_else(|| attr::u32(element, "width"))
            .unwrap_or(300),
        height: attr::u32(element, "data-agent-webgl-height")
            .or_else(|| attr::u32(element, "height"))
            .unwrap_or(150),
        viewport: attr::array(element, "data-agent-webgl-viewport", 0),
        clear_color: attr::array(element, "data-agent-webgl-clear-color", 0.0),
        scissor_box: attr::array(element, "data-agent-webgl-scissor-box", 0),
        scissor_test: attr::boolean(element, "data-agent-webgl-scissor-test"),
        color_mask: attr::array(element, "data-agent-webgl-color-mask", true),
        supported_extensions: attr::list(element, "data-agent-webgl-extensions"),
        commands: commands_from_attr(element),
    })
}

fn commands_from_attr(element: &Element) -> Vec<WebGlCommand> {
    element
        .attrs
        .get("data-agent-webgl-commands")
        .map(|raw| raw.split(';').filter_map(parse_command).collect())
        .unwrap_or_default()
}
