//! Dependency-free HTML template built-ins.
//!
//! the reference application renders every page through Tera, but tetherscript's Tera support sits
//! behind the optional `tera` feature, so the default zero-dependency build cannot render
//! a page at all. This group provides a small always-available renderer so the port can
//! serve HTML without a dependency.
//!
//! It covers the Tera subset the reference views actually lean on: substitution,
//! `if`/`elif`/`else`, `for`, `extends`/`block` inheritance, `include`, comments, and the
//! common filters. It is still not a Tera clone — `macro` and `set` are absent, and each
//! is rejected by name. Use `tera_render` when the `tera` feature is enabled and those
//! are needed.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `html_escape(text)` | escaped text |
//! | `html_attr(text)` | text escaped for an attribute value |
//! | `template_render(template, context)` | `Result` of rendered text, escaping on |
//! | `template_render_raw(template, context)` | the same with escaping off |
//! | `template_render_inherited(template, context, templates)` | resolves `extends` and `include` |
//!
//! # Examples
//!
//! ```text
//! let ctx = map()
//! ctx.title = "Bins & Cans"
//! println(template_render("<h1>\{\{ title \}\}</h1>", ctx).unwrap())
//! ```
//!
//! Submodules live in `template/`, so the declarations below need no `#[path]`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

pub(super) mod template_block;
pub(super) mod template_blocks;
pub(super) mod template_branch;
pub(super) mod template_condition;
pub(super) mod template_context;
pub(super) mod template_delimit;
pub(super) mod template_emit;
pub(super) mod template_escape;
pub(super) mod template_extends;
pub(super) mod template_filter;
pub(super) mod template_filter_arg;
pub(super) mod template_filter_civil;
pub(super) mod template_filter_date;
pub(super) mod template_filter_fn;
pub(super) mod template_filter_len;
pub(super) mod template_filter_list;
pub(super) mod template_filter_month;
pub(super) mod template_filter_split;
pub(super) mod template_filter_strftime;
pub(super) mod template_filter_text;
pub(super) mod template_filter_truncate_args;
pub(super) mod template_include;
pub(super) mod template_inherit;
pub(super) mod template_install;
pub(super) mod template_install_escape;
pub(super) mod template_install_render;
pub(super) mod template_loop;
pub(super) mod template_render;
pub(super) mod template_scan;
pub(super) mod template_source;
pub(super) mod template_step;
pub(super) mod template_subject;
pub(super) mod template_tag;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    template_install::install(env);
}
