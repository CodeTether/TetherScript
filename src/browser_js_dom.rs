//! Focused DOM compatibility host shims.

use super::*;

#[allow(dead_code)]
#[path = "browser_js_dom/attr_update.rs"]
mod attr_update;
#[path = "browser_js_dom/character_data.rs"]
mod character_data;
#[path = "browser_js_dom/construct.rs"]
mod construct;
#[path = "browser_js_dom/convenience.rs"]
pub(in crate::browser_js) mod convenience;
#[path = "browser_js_dom/dialog/mod.rs"]
mod dialog;
#[path = "browser_js_dom/document.rs"]
pub(in crate::browser_js) mod document;
#[path = "browser_js_dom/file_input/mod.rs"]
mod file_input;
#[path = "browser_js_dom/form_validation/mod.rs"]
pub(super) mod form_validation;
#[path = "browser_js_dom/install.rs"]
mod install;
#[path = "browser_js_dom/observer/mod.rs"]
pub(crate) mod observer;
#[path = "browser_js_dom/ops.rs"]
mod ops;
#[path = "browser_js_dom/parser.rs"]
mod parser;
#[path = "browser_js_dom/popover/mod.rs"]
mod popover;
#[path = "browser_js_dom/serializer.rs"]
mod serializer;
#[path = "browser_js_dom/template.rs"]
mod template;
#[path = "browser_js_dom/traversal/mod.rs"]
mod traversal;

pub(super) use install::{install_character_data, install_live_node, install_node, install_window};
