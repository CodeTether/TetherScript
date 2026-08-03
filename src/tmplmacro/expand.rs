//! The integration boundary: a pure `expand` that returns what the engine needs to render.
//!
//! # Why this does not render
//!
//! The engine must remain the **single renderer**. If this component rendered a macro body
//! it would need its own copy of hole emission, escaping, filters, and tag dispatch, and the
//! two renderers would drift — a `| safe` honoured in one and not the other is an XSS hole.
//! So [`expand`] returns an [`Expansion`]: the body **source** and the child **context**.
//! The engine renders that body with that context using its existing `render_with`, and
//! escaping, filters, and inheritance are inherited unchanged.
//!
//! # What crosses the boundary
//!
//! * In: the [`Registry`] of reachable macros, the hole body text, the caller's context, and
//!   the active [`Frames`] stack.
//! * Out: body source, child context, and the *next* frame stack to pass down when the body
//!   itself contains a macro call. That is how nesting works without this file recursing.

use crate::tmplmacro::bind::bind;
use crate::tmplmacro::call::parse_call;
use crate::tmplmacro::frames::Frames;
use crate::tmplmacro::lookup::resolve_call;
use crate::tmplmacro::registry::Registry;
use crate::value::Value;

/// What the engine needs in order to render one macro call.
#[derive(Debug, Clone)]
pub struct Expansion {
    /// Body source text of the macro, verbatim from its definition.
    pub body: String,
    /// Child context: exactly the macro's parameters, and nothing from the caller.
    pub context: Value,
    /// Frame stack to pass when expanding calls found inside `body`.
    pub frames: Frames,
}

/// Expand one macro call into the body and context the engine should render.
///
/// # Arguments
///
/// * `registry` — Macros reachable from the template being rendered.
/// * `body` — Trimmed hole body, such as `booking::service_calendar(cfg=x)`.
/// * `caller` — The calling context; read only to evaluate argument expressions.
/// * `frames` — Active expansion stack; [`Frames::new`] at the outermost call.
///
/// # Returns
///
/// An [`Expansion`]. Nothing is rendered and nothing is mutated.
///
/// # Errors
///
/// Returns an error for a malformed call, an unknown namespace or name, a positional
/// argument, an unknown parameter name, a missing required argument, an argument expression
/// that does not resolve, or recursion caught by [`Frames::push`].
///
/// # Panics
///
/// None.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::expand::expand;
/// use tetherscript::tmplmacro::frames::Frames;
/// use tetherscript::tmplmacro::macros::collect;
/// use tetherscript::tmplmacro::registry::Registry;
/// use tetherscript::value::Value;
///
/// let ui = collect(r#"{% macro badge(kind, size="sm") %}<b>{{ kind }}</b>{% endmacro %}"#)
///     .unwrap();
/// let registry = Registry::default().with("ui", ui);
///
/// let out = expand(&registry, r#"ui::badge(kind="new")"#, &Value::Nil, &Frames::new())
///     .unwrap();
/// assert_eq!(out.body, "<b>{{ kind }}</b>");
/// assert_eq!(out.frames.depth(), 1);
///
/// let Value::Map(map) = out.context else { panic!("expected a map") };
/// assert_eq!(map.borrow().len(), 2);
/// ```
pub fn expand(
    registry: &Registry,
    body: &str,
    caller: &Value,
    frames: &Frames,
) -> Result<Expansion, String> {
    let call = parse_call(body)?;
    let (def, key) = resolve_call(registry, &call)?;
    let next = frames.push(&key)?;
    let context = bind(def, call.path, &call.args, caller)?;
    Ok(Expansion {
        body: def.body.clone(),
        context,
        frames: next,
    })
}
