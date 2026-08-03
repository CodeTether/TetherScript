//! Inheritance chain resolution.
//!
//! Walks `{% extends %}` from the leaf template to the root, accumulating block
//! overrides. The leaf wins: a block defined in the child replaces the parent's,
//! which is the whole point of inheritance.

use std::collections::HashMap;

use super::template_blocks::{collect, Blocks};
use super::template_extends::parent_of;
use super::template_extends::source_of;
use super::template_scan::scan;
use super::template_source::to_source;
use crate::value::Value;

/// A resolved chain: the root template's source plus the winning block bodies.
pub(super) struct Resolved {
    /// Source of the outermost ancestor, which is what actually gets rendered.
    pub root: String,
    /// Block overrides, keyed by name, rendered as raw source text.
    pub overrides: HashMap<String, String>,
}

/// Depth limit for the inheritance chain.
///
/// A cycle would otherwise loop until the process ran out of memory. Real
/// hierarchies are two or three deep, so this is generous.
const MAX_DEPTH: usize = 16;

/// Follow `extends` from `template` to its root ancestor.
///
/// # Arguments
///
/// * `template` — Leaf template source.
/// * `templates` — Map of name to source for ancestors.
///
/// # Returns
///
/// The root source and the accumulated block overrides.
///
/// # Errors
///
/// Returns an error for a missing parent, a malformed `extends`, or a chain deeper
/// than [`MAX_DEPTH`], which is how a cycle is reported.
pub(super) fn resolve(template: &str, templates: &Value) -> Result<Resolved, String> {
    let mut overrides: HashMap<String, String> = HashMap::new();
    let mut source = template.to_string();
    for _ in 0..MAX_DEPTH {
        let pieces = scan(&source)?;
        // A child's blocks are only recorded if an ancestor does not already define
        // an override, so the most-derived template wins.
        merge(&mut overrides, collect(&pieces)?);
        match parent_of(&pieces)? {
            Some(name) => source = source_of(templates, name)?,
            None => {
                return Ok(Resolved {
                    root: source,
                    overrides,
                })
            }
        }
    }
    Err(format!(
        "template: inheritance chain deeper than {MAX_DEPTH}; `extends` may be cyclic"
    ))
}

/// Record blocks that no more-derived template already claimed.
fn merge(overrides: &mut HashMap<String, String>, blocks: Blocks<'_>) {
    for (name, pieces) in blocks {
        if overrides.contains_key(&name) {
            continue;
        }
        overrides.insert(name, to_source(&pieces));
    }
}
