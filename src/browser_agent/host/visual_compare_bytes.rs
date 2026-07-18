//! Encoded screenshot byte-difference counting.

pub(super) fn changed(before: &[u8], after: &[u8]) -> usize {
    let shared = before
        .iter()
        .zip(after)
        .filter(|(left, right)| left != right)
        .count();
    shared + before.len().abs_diff(after.len())
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts_changed_and_missing_bytes() {
        assert_eq!(super::changed(&[1, 2, 3], &[1, 4]), 2);
    }
}
