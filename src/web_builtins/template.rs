//! Dependency-free HTML template built-ins.
//!
//! the reference application renders every page through Tera, but tetherscript's Tera support
//! sits behind the optional `tera` feature, so the default zero-dependency build
//! cannot render a page at all. This group provides a small always-available
//! renderer so the port can serve HTML without a dependency. It is deliberately
//! not a Tera clone: there are no filters, loops, conditionals, or inheritance.
//! Use `tera_render` when the `tera` feature is enabled and those are needed.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `html_escape(text)` | escaped str |
//! | `html_attr(text)` | str safe inside a quoted attribute |
//! | `template_render(template, context)` | `Result` of rendered str, escaping by default |
//! | `template_render_raw(template, context)` | `Result` of rendered str, no escaping |
//!
//! # Security
//!
//! **`{{ name }}` escapes; `{{{ name }}}` does not.** Escaping by default is the
//! entire point: a renderer that interpolates untrusted values verbatim is an XSS
//! vector, so opting out has to be explicit and visible in the template. Reach for
//! the triple-brace form only for markup you generated yourself.
//!
//! An unknown key is a named `Err`, never an empty string, so a typo cannot
//! quietly blank part of a page. `template_render_raw` escapes nothing and is
//! intended for non-HTML output such as plain text or CSV.
//!
//! # Examples
//!
//! ```tether
//! let ctx = map()
//! ctx.title = "Hello & welcome"
//! // Renders: <h1>Hello &amp; welcome</h1>
//! println(template_render("<h1>\{\{ title \}\}</h1>", ctx).unwrap())
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "template_args.rs"]
pub(super) mod template_args;
#[path = "template_block.rs"]
pub(super) mod template_block;
#[path = "template_block_tag.rs"]
pub(super) mod template_block_tag;
#[path = "template_blocks.rs"]
pub(super) mod template_blocks;
#[path = "template_bounds.rs"]
pub(super) mod template_bounds;
#[path = "template_context.rs"]
pub(super) mod template_context;
#[path = "template_delimit.rs"]
pub(super) mod template_delimit;
#[path = "template_escape.rs"]
pub(super) mod template_escape;
#[path = "template_extends.rs"]
pub(super) mod template_extends;
#[path = "template_inherit.rs"]
pub(super) mod template_inherit;
#[path = "template_install.rs"]
pub(super) mod template_install;
#[path = "template_lookup.rs"]
pub(super) mod template_lookup;
#[path = "template_loop.rs"]
pub(super) mod template_loop;
#[path = "template_loop_header.rs"]
pub(super) mod template_loop_header;
#[path = "template_render.rs"]
pub(super) mod template_render;
#[path = "template_scan.rs"]
pub(super) mod template_scan;
#[path = "template_step.rs"]
pub(super) mod template_step;
#[path = "template_subject.rs"]
pub(super) mod template_subject;
#[path = "template_tag.rs"]
pub(super) mod template_tag;
#[path = "template_tag_unknown.rs"]
pub(super) mod template_tag_unknown;
#[path = "template_truth.rs"]
pub(super) mod template_truth;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    template_install::install(env);
}
