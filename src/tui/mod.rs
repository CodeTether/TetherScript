//! Dependency-free terminal UI helpers for scripts.

mod ansi;
mod buffer;
mod diff;
mod escape;
mod fields;
mod input;
mod input_state;
mod install;
mod jsonrpc;
mod jsonrpc_error;
mod key_csi;
mod key_event;
mod key_parse;
mod key_read;
mod key_value;
mod line;
mod line_item;
mod native;
mod panel;
mod panel_rows;
mod panel_state;
mod render;
mod scroll_state;
mod size;
mod status_bar;
mod stdio_err;
mod stdio_io;
mod style;
mod style_attr;
mod style_color;
mod style_open;
mod style_span;
mod val;
mod view;
mod view_extra;
mod view_input;

pub(crate) use native::document as native_document;

pub(super) fn install(env: &mut crate::value::Env) {
    install::install(env);
}

#[cfg(test)]
mod key_tests;
#[cfg(test)]
mod stdio_tests;
#[cfg(test)]
mod style_tests;
#[cfg(test)]
mod tests;
