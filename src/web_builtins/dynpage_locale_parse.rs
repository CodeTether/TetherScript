//! `Accept-Language` parsing, with a hard bound.
//!
//! # Attacker-controlled and therefore bounded
//!
//! `Accept-Language` is a client header of unlimited length, and a parser that
//! sorts every entry hands an attacker an `O(n log n)` amplifier for free: one
//! request carrying tens of thousands of `xx;q=0.5` entries becomes real CPU on
//! every hop that negotiates. Two bounds close that.
//!
//! * [`MAX_ENTRIES`] — at most **16** comma-separated entries are parsed. Entries
//!   past the bound are discarded, not rejected: an over-long header is still a
//!   valid request, and the leading entries are the ones a real client considers
//!   most important. Real browsers send fewer than five.
//! * [`MAX_CHARS`] — at most **512** characters of the header are examined at all,
//!   so neither the split nor the scan can be made expensive before the entry
//!   bound applies. The cut is taken at a character boundary via `char_indices`,
//!   never at a raw byte index, because slicing a `str` mid-character panics and
//!   the header is attacker-controlled.
//!
//! # Never echoed
//!
//! Nothing parsed here reaches a key, a header, or an error message. A tag is only
//! ever *compared* against the caller's supported list, and the value returned is
//! an element of that list. So a header carrying markup, a newline, or a separator
//! byte cannot be reflected into a response or into a cache key.
//!
//! # Malformed entries
//!
//! A non-numeric or out-of-range `q` is treated as a quality of 0, which drops the
//! entry — not as the RFC default of 1. Defaulting garbage to the *highest*
//! priority would let a malformed entry outrank a well-formed one. An entry with no
//! `q` parameter at all still gets the RFC default of 1.

/// Most comma-separated entries parsed from one header.
pub(super) const MAX_ENTRIES: usize = 16;

/// Most header characters examined.
const MAX_CHARS: usize = 512;

/// One parsed language range.
pub(super) struct Entry {
    /// Lower-cased language tag, or `*`.
    pub(super) tag: String,
    /// Quality in thousandths, so ordering is integer and exact.
    pub(super) quality: u32,
}

/// Parse an `Accept-Language` header into ranges, best first.
///
/// # Arguments
///
/// * `header` — Raw header value.
///
/// # Returns
///
/// At most [`MAX_ENTRIES`] entries of non-zero quality, sorted by descending
/// quality. The sort is stable, so equal qualities keep the client's stated order,
/// which the RFC treats as significant.
pub(super) fn parse(header: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = bounded(header)
        .split(',')
        .take(MAX_ENTRIES)
        .filter_map(entry)
        .collect();
    // Descending by quality, so the client's most-preferred language comes first. Keyed on
    // the negated quality because `sort_by_key` cannot express a reversed comparator.
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.quality));
    entries
}

/// Truncate to [`MAX_CHARS`] at a character boundary.
fn bounded(header: &str) -> &str {
    match header.char_indices().nth(MAX_CHARS) {
        Some((index, _)) => &header[..index],
        None => header,
    }
}

/// Parse one comma-separated range, dropping it when unusable.
fn entry(part: &str) -> Option<Entry> {
    let mut pieces = part.split(';');
    let tag = pieces.next()?.trim().to_ascii_lowercase();
    if tag.is_empty() {
        return None;
    }
    let quality = pieces.find_map(quality_of).unwrap_or(1000);
    (quality > 0).then_some(Entry { tag, quality })
}

/// Read a `q=` parameter as thousandths, or 0 when it is malformed.
fn quality_of(param: &str) -> Option<u32> {
    let value = param.trim().strip_prefix("q=")?;
    Some(match value.trim().parse::<f64>() {
        Ok(number) if (0.0..=1.0).contains(&number) => (number * 1000.0).round() as u32,
        _ => 0,
    })
}
