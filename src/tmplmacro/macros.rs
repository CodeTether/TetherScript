//! Macro definition collection: the primary entry point of this component.
//!
//! Given raw template source, find every `{% macro name(params) %}...{% endmacro %}` and
//! record its name, parameter list with defaults, and **body source text**. The body is
//! kept as a source slice, not a rendered string and not a parsed piece list, because
//! [`crate::tmplmacro::expand::expand`] must hand it back to the engine for the engine to
//! render — this component is never a second renderer.
//!
//! # Semantics fixed here
//!
//! * **Definitions are hoisted.** A `macro` nested inside `{% if %}` or `{% for %}` is
//!   still collected, because a definition is a declaration, not conditional output.
//! * **Duplicates are rejected, not last-wins.** Two `{% macro row %}` in one namespace is
//!   a mistake; resolving it to the later one ships whichever component the author did not
//!   meant, with no diagnostic, so [`collect`] returns an error instead.
//! * **Both end forms are accepted.** `{% endmacro %}` and `{% endmacro name %}`.
//! * **A definition emits nothing.** Reaching a `macro` tag while rendering skips to just
//!   past its `endmacro`; only a call produces text.

use std::collections::BTreeMap;

use crate::tmplmacro::endmatch::matching_end;
use crate::tmplmacro::params::parse_header;
use crate::tmplmacro::params_item::Param;
use crate::tmplmacro::tags::tags_of;

/// One collected macro definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDef {
    /// Macro name as declared.
    pub name: String,
    /// Declared parameters, in source order, each with an optional raw default.
    pub params: Vec<Param>,
    /// Body source text between the `macro` tag and its `endmacro`, verbatim.
    pub body: String,
}

/// Every macro one template defines, keyed by name.
///
/// `BTreeMap` rather than `HashMap` so iteration order is deterministic, which keeps
/// error messages and test assertions stable.
pub type MacroSet = BTreeMap<String, MacroDef>;

/// Collect every macro defined by `source`.
///
/// # Arguments
///
/// * `source` — Raw template text.
///
/// # Returns
///
/// A name-to-definition map, empty when the template defines no macros.
///
/// # Errors
///
/// Returns an error for a malformed header, an unbalanced `endmacro`, or a duplicate
/// macro name within this one source.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::macros::collect;
///
/// let src = r#"{% macro badge(kind, size="sm") %}<b>{{ kind }}</b>{% endmacro badge %}"#;
/// let set = collect(src).unwrap();
/// assert_eq!(set["badge"].params[1].default.as_deref(), Some("\"sm\""));
/// assert_eq!(set["badge"].body, "<b>{{ kind }}</b>");
/// ```
pub fn collect(source: &str) -> Result<MacroSet, String> {
    let tags = tags_of(source);
    let mut set = MacroSet::new();
    for (index, tag) in tags.iter().enumerate() {
        if tag.keyword() != "macro" {
            continue;
        }
        let (name, params) = parse_header(tag.body)?;
        let end = matching_end(&tags, index)?;
        let body = source[tag.end..tags[end].start].to_string();
        if set.contains_key(&name) {
            return Err(format!(
                "template: macro `{name}` is defined twice in one template; \
                 rename one — a later definition does not silently replace the earlier"
            ));
        }
        set.insert(name.clone(), MacroDef { name, params, body });
    }
    Ok(set)
}
