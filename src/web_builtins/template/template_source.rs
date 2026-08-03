//! Piece-to-source reconstruction.
//!
//! A block's body must be re-scanned in the context of whichever template ends up
//! rendering it, and reconstructing text is simpler than threading byte offsets through
//! every layer.

use super::template_scan::Piece;

/// Re-render pieces to equivalent source text.
pub(super) fn to_source(pieces: &[Piece<'_>]) -> String {
    pieces.iter().map(one_source).collect()
}

/// Source text for a single piece.
fn one_source(piece: &Piece<'_>) -> String {
    match piece {
        Piece::Text(text) => (*text).to_string(),
        Piece::Escaped(name) => format!("{{{{ {name} }}}}"),
        Piece::Raw(name) => format!("{{{{{{ {name} }}}}}}"),
        Piece::Tag(body) => format!("{{% {body} %}}"),
        // The body was discarded at scan time, so an empty comment round-trips.
        Piece::Comment => "{##}".to_string(),
    }
}
