//! Macro call-site detection and dispatch.
//!
//! The reference application calls a macro through a hole, not a tag:
//! `{{ booking::service_calendar(arg=value) }}`. So this is reached from expression
//! emission via [`super::template_macro_hole`], not from tag dispatch.
//!
//! # How a namespace resolves
//!
//! Templates come from a caller-supplied map, not the filesystem, so there is no path to
//! import. A namespace is therefore simply **a key in that map**: `ui::badge(...)` means
//! "the macro `badge` defined by the template stored under the key `ui`". That makes
//! `{% import %}` unnecessary — the aliasing it exists to provide is already the map key.
//! `self::name(...)`, `_self::name(...)`, and bare `name(...)` all resolve against the
//! template currently being rendered, which is what lets a macro recurse.
//!
//! # Recursion
//!
//! Bounded at 16 nested calls, the same limit and the same shared counter
//! [`super::template_block::Render::depth`] that [`super::template_include`] uses. A
//! runaway macro is reported as an error rather than overflowing the stack and aborting
//! the process.

use super::template_block::Render;
use super::template_extends::source_of;
use super::template_macro::collect;
use super::template_macro_invoke::invoke;
use super::template_macro_path::{split_call, split_path};
use super::template_scan::{scan, Piece};
use crate::value::Value;

/// Maximum nesting depth for macro calls and includes combined.
const MAX_DEPTH: usize = 16;

/// Whether a hole body is a macro call rather than a value expression.
///
/// # Arguments
///
/// * `body` — Trimmed text inside `{{ }}`.
///
/// # Returns
///
/// True when the body is `path(...)` and `path` is a bare identifier path of
/// alphanumerics, `_`, and `:`. A filter pipeline such as `x | default(value="")` is
/// excluded, because its head contains a space and a `|`.
///
/// # Errors
///
/// None; this is a classification, so an unparseable body is left to the value path,
/// which already reports malformed expressions.
///
/// # Examples
///
/// ```text
/// is_call("ui::badge(kind=\"new\")")  →  true
/// is_call("cfg.html | safe")          →  false
/// ```
pub(super) fn is_call(body: &str) -> bool {
    let body = body.trim();
    let Some(open) = body.find('(') else {
        return false;
    };
    let head = &body[..open];
    let bare = head
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ':');
    body.ends_with(')') && !head.is_empty() && bare
}

/// Render a macro call.
///
/// # Arguments
///
/// * `pieces` — Pieces of the template being rendered, so a same-template or recursive
///   call can find its own definition.
/// * `body` — Trimmed hole body, such as `ui::badge(kind="new")`.
/// * `context` — Caller context, used only to evaluate argument expressions.
/// * `state` — Current render state, supplying the template map and depth counter.
///
/// # Returns
///
/// The macro body rendered against a scope holding only its parameters.
///
/// # Errors
///
/// Returns an error for a malformed call, an unknown namespace or macro, a missing
/// required argument, an unknown keyword argument, or nesting past [`MAX_DEPTH`].
pub(super) fn call(
    pieces: &[Piece<'_>],
    body: &str,
    context: &Value,
    state: &Render<'_>,
) -> Result<String, String> {
    if state.depth >= MAX_DEPTH {
        return Err(format!(
            "template: macro call `{body}` nested deeper than {MAX_DEPTH}; \
             a macro may call itself"
        ));
    }
    let (path, args) = split_call(body)?;
    let (namespace, name) = split_path(path);
    match namespace {
        Some(namespace) => {
            // Re-scanned per call rather than cached, since the render state this change
            // may not modify has nowhere to hold a macro table.
            let source = source_of(state.templates, namespace)?;
            let defined = scan(&source)?;
            invoke(&collect(&defined)?, namespace, name, args, context, state)
        }
        None => invoke(&collect(pieces)?, "self", name, args, context, state),
    }
}
