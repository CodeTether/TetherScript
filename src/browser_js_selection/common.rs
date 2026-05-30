use super::*;

/// Shared DOM handle utilities for range and selection operations.
///
/// Provides helper functions that compute common properties of DOM ranges,
/// such as the nearest shared ancestor node between two boundary points.
///
/// Computes the nearest common ancestor [`DomHandle`] for a DOM range.
///
/// Walks the path segments of the range's start and end handles in lockstep,
/// collecting segments while they are equal. The resulting handle points to the
/// deepest node that is an ancestor of both boundary points.
///
/// # Parameters
///
/// * `range` — A [`state::RangeState`] whose start and end handles define the
///   two boundary points to compare.
///
/// # Returns
///
/// A [`DomHandle`] whose path is the longest common prefix of the start and
/// end handle paths. The root is preserved from the range's handles (they are
/// expected to share the same root).
///
/// # Examples
///
/// If the start handle has path `[0, 1, 2]` and the end handle has path
/// `[0, 1, 4]`, the result is a handle with path `[0, 1]` — the parent
/// element containing both boundary points.
pub(super) fn ancestor(range: &state::RangeState) -> DomHandle {
    let mut path = Vec::new();
    for (left, right) in range.start.handle.path.iter().zip(&range.end.handle.path) {
        if left != right {
            break;
        }
        path.push(*left);
    }
    DomHandle {
        root: range.start.handle.root.clone(),
        path,
    }
}
