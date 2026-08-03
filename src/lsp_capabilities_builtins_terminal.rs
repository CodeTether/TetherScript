//! Terminal, TUI, stdio, and JSON-RPC builtins.
//!
//! Ported from `editor/vscode/lib/tool-data-terminal.js`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_terminal::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "tui_render"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Terminal builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("jsonrpc_error", "id, code, message[, data]", "Create a JSON-RPC error response."),
    ("jsonrpc_notify", "method, params", "Create a JSON-RPC notification."),
    ("jsonrpc_request", "id, method, params", "Create a JSON-RPC request."),
    ("jsonrpc_response", "id, result", "Create a JSON-RPC success response."),
    ("stdio_read", "", "Read one framed standard-input message."),
    ("stdio_write", "value", "Write one framed standard-output message."),
    ("stdio_write_err", "text", "Write text to standard error."),
    ("tui_alt_screen", "enabled", "Return an alternate-screen control sequence."),
    ("tui_clear", "", "Return a terminal clear-screen sequence."),
    ("tui_cursor", "visible", "Return a cursor visibility sequence."),
    ("tui_enter", "", "Return the TUI entry control sequence."),
    ("tui_leave", "", "Return the TUI exit control sequence."),
    ("tui_move_to", "row, col", "Return a cursor-position sequence."),
    ("tui_present", "view", "Clear and draw a terminal UI view."),
    ("tui_read_event", "[prompt]", "Read one terminal input event."),
    ("tui_read_key", "", "Read one parsed terminal key event."),
    ("tui_render", "view", "Render a terminal UI view to text."),
    ("tui_size", "", "Return terminal rows and columns."),
    ("tui_span_render", "span", "Render one styled text span."),
    ("tui_style_open", "style", "Return the ANSI sequence for a style."),
    ("tui_style_reset", "", "Return the ANSI style reset sequence."),
];
