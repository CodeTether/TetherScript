//! Browser and JavaScript-track builtins.
//!
//! Ported from `editor/vscode/lib/tool-data-browser.js`. These belong to the
//! in-tree experimental browser subset described in AGENTS.md, not to a wrapped
//! external engine.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::builtins_browser::TABLE;
//! assert!(TABLE.iter().any(|entry| entry.0 == "js_eval"));
//! ```

use crate::lsp_capabilities::builtins::Entry;

/// Browser builtins as `(name, params, summary)` rows.
#[rustfmt::skip]
pub const TABLE: &[Entry] = &[
    ("browser_compatibility_report", "", "Report supported browser runtime features."),
    ("browser_display_list", "html[, css[, width]]", "Build a browser paint display list."),
    ("browser_eval_js", "html, script", "Evaluate JavaScript against an HTML document."),
    ("browser_layout", "html[, css[, width]]", "Compute browser layout boxes."),
    ("browser_parse_css", "css", "Parse CSS into runtime values."),
    ("browser_parse_html", "html", "Parse HTML into runtime values."),
    ("browser_query_selector", "html, selector", "Query the first matching HTML element."),
    ("browser_raster", "html[, css[, width]]", "Rasterize a browser document."),
    ("browser_render", "html[, css[, width]]", "Render a browser document."),
    ("browser_render_ppm", "html[, css[, width]]", "Render a browser document as PPM bytes."),
    ("browser_run_scripts", "html", "Run scripts embedded in an HTML document."),
    ("browser_run_scripts_at", "html, url", "Run embedded scripts with a document URL."),
    ("browser_snapshot", "html[, css[, width]]", "Create a browser document snapshot."),
    ("browser_styles", "html[, css]", "Compute styles for an HTML document."),
    ("browser_text_content", "html, selector", "Return text from the first matching element."),
    ("js_eval", "source", "Evaluate JavaScript and return a tetherscript value."),
];
