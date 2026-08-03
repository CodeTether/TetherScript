//! Depth-tracked discovery of the `{% endmacro %}` closing a definition.
//!
//! Mirrors the rule stated in the engine's `template_delimit::matching_end`: an inner
//! block must not terminate an outer one early. A macro body routinely contains
//! `{% if %}`/`{% for %}`, and a macro may be *defined* inside an `{% if %}`, so every
//! opener counts toward depth and every closer counts back down.
//!
//! Both `{% endmacro %}` and `{% endmacro name %}` close a definition. The trailing name
//! is a reader's annotation, exactly as `{% endblock name %}` is in the engine, so it is
//! not required to match the opener.

use crate::tmplmacro::tags::Tag;

/// Tag keywords that open a depth level.
const OPENERS: [&str; 4] = ["if", "for", "block", "macro"];

/// Tag keywords that close a depth level.
const CLOSERS: [&str; 4] = ["endif", "endfor", "endblock", "endmacro"];

/// Index of the tag closing the block opened at `open`.
///
/// # Arguments
///
/// * `tags` — All tags of one template, in source order.
/// * `open` — Index of the opening tag; its keyword must be an opener.
///
/// # Returns
///
/// The index into `tags` of the matching closer.
///
/// # Errors
///
/// Returns an error naming the unclosed opener's body when depth never returns to zero.
///
/// # Panics
///
/// None. `open` out of range yields the unclosed error rather than a panic.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::endmatch::matching_end;
/// use tetherscript::tmplmacro::tags::tags_of;
///
/// let src = "{% macro a() %}{% if x %}{% endif %}{% endmacro %}";
/// let tags = tags_of(src);
/// assert_eq!(matching_end(&tags, 0).unwrap(), 3);
/// ```
pub fn matching_end(tags: &[Tag<'_>], open: usize) -> Result<usize, String> {
    let opener = tags.get(open).map_or("<none>", |tag| tag.body);
    let mut depth = 0usize;
    for (offset, tag) in tags.iter().enumerate().skip(open) {
        let word = tag.keyword();
        if OPENERS.contains(&word) {
            depth += 1;
        } else if CLOSERS.contains(&word) {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Ok(offset);
            }
        }
    }
    Err(format!(
        "template: `{{% {opener} %}}` is never closed; expected a matching `endmacro`"
    ))
}
