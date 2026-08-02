//! Rejection of tags the engine does not implement.
//!
//! Each unsupported tag is named individually so a ported template fails with a
//! message a reader can act on, rather than rendering a silent hole. Split out so
//! the guidance can grow without pushing the dispatcher over the line budget.

/// Reject an unsupported tag keyword.
///
/// # Errors
///
/// Always returns an error; the return type matches the dispatcher's.
pub(super) fn reject(keyword: &str) -> Result<usize, String> {
    let hint = match keyword {
        "include" => " — inline the partial, or render it separately and pass the result",
        "macro" | "endmacro" | "import" => " — define a helper fn in tetherscript instead",
        "set" => " — compute the value in tetherscript and pass it in the context",
        "raw" | "endraw" | "filter" | "endfilter" => "",
        _ => "",
    };
    Err(format!(
        "template: unsupported tag `{keyword}`{hint} (have: if, else, endif, for, endfor, \
         block, endblock, extends)"
    ))
}
