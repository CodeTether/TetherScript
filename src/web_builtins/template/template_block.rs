//! Render state threaded through every evaluation layer.
//!
//! Bundled into one [`Render`] value rather than passed as separate parameters: `include` needs the
//! template map and a depth counter, and threading those through every signature individually would
//! push several files over the line budget.
//!
//! The `nested` and `tolerant` builders return a fresh value rather than mutating, so a nested
//! render cannot alter its parent's state — an include one level down must not leave escaping
//! disabled for whatever follows it.

use std::collections::HashMap;

use crate::value::Value;

/// Everything an evaluation step needs besides the pieces and the context.
pub(super) struct Render<'a> {
    /// Whether `{{ }}` output is HTML-escaped.
    pub escaping: bool,
    /// Block bodies supplied by a more-derived template.
    pub overrides: &'a HashMap<String, String>,
    /// Map of template name to source, for `extends` and `include`.
    pub templates: &'a Value,
    /// Current include nesting depth, for cycle detection.
    pub depth: usize,
    /// When true, an unknown key renders as empty instead of failing the render.
    ///
    /// Off by default, because a typo like `{{ user.nmae }}` should be caught rather than shipped as
    /// a blank. On for a caller rendering templates it did not author: Tera's own default is
    /// lenient, so a large view tree written against it references keys a port has no equivalent
    /// for, and one of them must not take a whole page down.
    pub lenient: bool,
}

impl<'a> Render<'a> {
    /// State for a render whose templates and overrides the caller owns.
    pub(super) fn new(
        escaping: bool,
        overrides: &'a HashMap<String, String>,
        templates: &'a Value,
    ) -> Self {
        Self {
            escaping,
            overrides,
            templates,
            depth: 0,
            lenient: false,
        }
    }

    /// The same state, optionally one include deeper and with escaping overridden.
    pub(super) fn nested(&self, escaping: bool, deeper: bool) -> Render<'a> {
        Render {
            escaping,
            overrides: self.overrides,
            templates: self.templates,
            depth: self.depth + usize::from(deeper),
            lenient: self.lenient,
        }
    }

    /// The same state with unknown keys rendering as empty.
    pub(super) fn tolerant(mut self) -> Self {
        self.lenient = true;
        self
    }
}

pub(super) use super::template_delimit::matching_end;
pub(super) use super::template_render_loop::render_with;
pub(super) use super::template_subject::iterable;
