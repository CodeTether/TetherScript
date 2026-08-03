//! Locating `{% ... %}` tags in raw template source.
//!
//! This component works on **source text**, not the engine's `Piece` list, for two
//! reasons. First, `template_scan::Piece` and `template_delimit::matching_end` are
//! `pub(super)` to the engine's template module and unreachable from here. Second, and
//! more importantly, [`crate::tmplmacro::expand`] must hand the engine back a **body
//! source string** for it to render with its own single renderer; retaining source spans
//! is therefore the natural representation rather than an accident of visibility.
//!
//! `{# comment #}` runs are skipped, because a commented-out `{% endmacro %}` must not
//! close a live definition.

/// One located tag: its trimmed body and the byte range of the whole `{% ... %}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'a> {
    /// Trimmed interior, e.g. `macro badge(kind)`.
    pub body: &'a str,
    /// Byte offset of the opening `{`.
    pub start: usize,
    /// Byte offset just past the closing `}`.
    pub end: usize,
}

impl Tag<'_> {
    /// First whitespace-separated word of the tag body, or `""`.
    ///
    /// # Returns
    ///
    /// The keyword, such as `macro`, `endmacro`, `if`, or `import`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::tmplmacro::tags::tags_of;
    ///
    /// let tags = tags_of("{% macro a() %}");
    /// assert_eq!(tags[0].keyword(), "macro");
    /// ```
    pub fn keyword(&self) -> &str {
        self.body.split_whitespace().next().unwrap_or("")
    }
}

/// Locate every `{% ... %}` tag in `source`, in order, skipping comments.
///
/// # Arguments
///
/// * `source` — Raw template text.
///
/// # Returns
///
/// The tags in source order. An unclosed `{%` is simply not reported, because the
/// engine's own scanner already errors on it and duplicating that error here would
/// report the same fault twice.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::tags::tags_of;
///
/// let tags = tags_of("a{% macro b() %}x{% endmacro %}");
/// assert_eq!(tags.len(), 2);
/// assert_eq!(tags[1].body, "endmacro");
/// assert_eq!(&"a{% macro b() %}x{% endmacro %}"[tags[0].end..tags[1].start], "x");
/// ```
pub fn tags_of(source: &str) -> Vec<Tag<'_>> {
    let (mut tags, mut at) = (Vec::new(), 0usize);
    while at < source.len() {
        let Some(rel) = source[at..].find('{') else { break };
        let open = at + rel;
        at = match next_span(source, open) {
            Some((body, end)) => {
                if let Some(body) = body {
                    tags.push(Tag { body, start: open, end });
                }
                end
            }
            None => open + 1,
        };
    }
    tags
}

/// Read the span starting at `open`; `Some((None, end))` for a comment.
fn next_span(source: &str, open: usize) -> Option<(Option<&str>, usize)> {
    let rest = &source[open..];
    if let Some(inner) = rest.strip_prefix("{#") {
        let end = inner.find("#}")?;
        return Some((None, open + 2 + end + 2));
    }
    let inner = rest.strip_prefix("{%")?;
    let end = inner.find("%}")?;
    Some((Some(inner[..end].trim()), open + 2 + end + 2))
}
