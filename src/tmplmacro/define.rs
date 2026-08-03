//! Definition-site handling: a `{% macro %}` tag emits nothing.
//!
//! Reaching a `{% macro %}` tag while rendering must produce no output at all — a
//! definition is a declaration, so a macro defined and never called contributes no text.
//! The engine therefore needs to know where to resume, which is just past the matching
//! `{% endmacro %}`.
//!
//! The header is parsed even though its result is discarded, so a typo in a macro that is
//! never called is still reported rather than lying in wait until the one page that uses it
//! is requested.

use crate::tmplmacro::endmatch::matching_end;
use crate::tmplmacro::params::parse_header;
use crate::tmplmacro::tags::tags_of;

/// Byte offset in `source` just past the `{% endmacro %}` closing the macro at `start`.
///
/// # Arguments
///
/// * `source` — Raw template text.
/// * `start` — Byte offset of the `{` opening the `{% macro %}` tag.
///
/// # Returns
///
/// The offset at which rendering should resume.
///
/// # Errors
///
/// Returns an error when `start` names no `macro` tag, the header is malformed, or the
/// definition is never closed.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::define::skip_definition;
///
/// let src = "a{% macro b() %}x{% endmacro %}z";
/// let at = skip_definition(src, 1).unwrap();
/// assert_eq!(&src[at..], "z");
/// ```
pub fn skip_definition(source: &str, start: usize) -> Result<usize, String> {
    let tags = tags_of(source);
    let index = tags
        .iter()
        .position(|tag| tag.start == start && tag.keyword() == "macro")
        .ok_or_else(|| format!("template: no `{{% macro %}}` tag at byte offset {start}"))?;
    parse_header(tags[index].body)?;
    Ok(tags[matching_end(&tags, index)?].end)
}
