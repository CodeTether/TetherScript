//! Macro call argument parsing.
//!
//! Split from dispatch so call syntax is one concern. Arguments are **keyword-only**, as
//! Tera's are: `row(cfg=item, wide=true)`. Positional arguments are deliberately *not*
//! supported — the reference's 85 call sites are all keyword form, and accepting position
//! would make `badge("new")` bind silently to whichever parameter happens to be first,
//! which is precisely the class of quiet breakage this engine reports instead.

use super::template_filter_split::split_outside_quotes;

/// One supplied argument: its keyword and the raw expression text for its value.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Arg<'a> {
    /// Keyword naming the parameter to bind.
    pub name: &'a str,
    /// Raw value expression: a literal, or a possibly-dotted context key.
    pub expression: &'a str,
}

/// Parse the comma-separated interior of a call's argument list.
///
/// # Arguments
///
/// * `arguments` — Text between the call's parentheses, already trimmed.
/// * `path` — Call path, used only to name the offender in an error.
///
/// # Returns
///
/// The arguments in source order. An empty list for `name()`.
///
/// # Errors
///
/// Returns an error for an argument with no `=`, since that is either a positional
/// argument or a typo and both must be reported, or for an empty keyword.
///
/// # Examples
///
/// ```text
/// parse("kind=\"new\", size=s", "ui::badge")  →  [kind, size]
/// ```
pub(super) fn parse<'a>(arguments: &'a str, path: &str) -> Result<Vec<Arg<'a>>, String> {
    if arguments.is_empty() {
        return Ok(Vec::new());
    }
    // Quote-aware, since a literal such as `sep=", "` legally contains a comma.
    split_outside_quotes(arguments, ',')
        .into_iter()
        .map(|part| one(part.trim(), path))
        .collect()
}

/// Parse a single `name=<expression>` argument.
fn one<'a>(text: &'a str, path: &str) -> Result<Arg<'a>, String> {
    let (name, expression) = text.split_once('=').ok_or_else(|| {
        format!(
            "template: argument `{text}` to macro `{path}` must be `name=value`; \
             macro arguments are keyword-only"
        )
    })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(format!(
            "template: empty argument name in call to macro `{path}`"
        ));
    }
    Ok(Arg {
        name,
        expression: expression.trim(),
    })
}
