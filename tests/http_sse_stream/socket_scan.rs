//! Byte scanning: substring search and head/body split.
//!
//! Written by hand rather than via `str` helpers because a stream body is not
//! guaranteed to be valid UTF-8 at an arbitrary read boundary, and lossy
//! conversion would silently repair exactly the byte defects under test.

/// Index of `needle` in `haystack`, byte-exact.
///
/// # Arguments
///
/// * `haystack` — Bytes to search.
/// * `needle` — Bytes to find.
///
/// # Returns
///
/// The first index, or `None` when absent or when `needle` is empty.
pub(crate) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|start| &haystack[*start..*start + needle.len()] == needle)
}

/// Split a response into its head, as text, and its body bytes.
///
/// # Arguments
///
/// * `response` — Raw bytes read from the socket.
///
/// # Returns
///
/// `(head, body)` where `head` excludes the terminating blank line.
///
/// # Panics
///
/// Panics when no `\r\n\r\n` is present: a response without a terminated head is
/// not something any assertion here can meaningfully interpret.
pub(crate) fn split(response: &[u8]) -> (String, Vec<u8>) {
    let at = find(response, b"\r\n\r\n").expect("head must be terminated by a blank line");
    let head = String::from_utf8_lossy(&response[..at]).to_string();
    (head, response[at + 4..].to_vec())
}
