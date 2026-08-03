//! `{% if %}` / `{% elif %}` / `{% else %}` branch location.
//!
//! Branches are located as index ranges rather than rendered eagerly, so only the taken
//! branch is evaluated — an untaken branch may legitimately reference keys that do not
//! exist, which is exactly how a view guards an optional value.

use super::template_delimit::Branch;
use super::template_scan::Piece;

/// Collect the branches of the `if` opened at `open`, plus its closing index.
///
/// Depth-tracked so an inner `if` does not contribute its own `elif`/`else`.
///
/// # Errors
///
/// Returns an error when the block is never closed, or a condition is missing.
pub(super) fn branches<'a>(
    pieces: &[Piece<'a>],
    open: usize,
) -> Result<(Vec<Branch<'a>>, usize), String> {
    let mut depth = 0usize;
    let mut found = Vec::new();
    for (offset, piece) in pieces.iter().enumerate().skip(open) {
        let Piece::Tag(body) = piece else { continue };
        // The condition is everything after the keyword, since a comparison such as
        // `step.id == current.id` spans several whitespace-separated words.
        let (keyword, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        match keyword {
            keyword @ ("if" | "for" | "block") => {
                depth += 1;
                if depth == 1 && keyword == "if" {
                    found.push(Branch {
                        condition: Some(non_empty(rest, "if")?),
                        at: offset,
                    });
                }
            }
            "elif" if depth == 1 => found.push(Branch {
                condition: Some(non_empty(rest, "elif")?),
                at: offset,
            }),
            "else" if depth == 1 => found.push(Branch {
                condition: None,
                at: offset,
            }),
            "endif" | "endfor" | "endblock" => {
                depth -= 1;
                if depth == 0 {
                    return Ok((found, offset));
                }
            }
            _ => {}
        }
    }
    Err("template: unbalanced `if`; missing `endif`".to_string())
}

/// Require a non-empty condition after `keyword`.
fn non_empty<'a>(rest: &'a str, keyword: &str) -> Result<&'a str, String> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        return Err(format!("template: `{keyword}` needs a condition"));
    }
    Ok(trimmed)
}
