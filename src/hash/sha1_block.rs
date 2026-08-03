//! Single-block SHA-1 compression function (RFC 3174 §6.1).
//!
//! Split out from `sha1.rs` so that the block function has exactly one
//! responsibility: mix one 64-byte chunk into the five-word state.

/// Expand a 64-byte chunk into the 80-word SHA-1 message schedule.
///
/// # Arguments
///
/// * `chunk` — Exactly 64 message bytes.
///
/// # Returns
///
/// The 80 big-endian-derived schedule words.
fn schedule(chunk: &[u8]) -> [u32; 80] {
    let mut w = [0u32; 80];
    for (slot, word) in w.iter_mut().take(16).zip(chunk.chunks_exact(4)) {
        *slot = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    w
}

/// Round function `f` and constant `k` for round `i`, per RFC 3174 §5.
fn round(i: usize, b: u32, c: u32, d: u32) -> (u32, u32) {
    match i {
        0..=19 => ((b & c) | ((!b) & d), 0x5a827999),
        20..=39 => (b ^ c ^ d, 0x6ed9eba1),
        40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
        _ => (b ^ c ^ d, 0xca62c1d6),
    }
}

/// Mix one 64-byte chunk into `h`.
///
/// # Arguments
///
/// * `h` — The five-word running state, updated in place.
/// * `chunk` — Exactly 64 message bytes.
pub(crate) fn compress(h: &mut [u32; 5], chunk: &[u8]) {
    let w = schedule(chunk);
    let [mut a, mut b, mut c, mut d, mut e] = *h;
    for (i, word) in w.iter().enumerate() {
        let (f, k) = round(i, b, c, d);
        let temp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(*word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }
    for (slot, value) in h.iter_mut().zip([a, b, c, d, e]) {
        *slot = slot.wrapping_add(value);
    }
}
