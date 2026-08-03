//! The SHA-1 block compression function (FIPS 180-4 §6.1.2).
//!
//! Kept separate from `sha1.rs` so padding/length handling and the round
//! function are independently readable, mirroring `src/postgres/md5_block.rs`.

/// Mix one 64-byte block into the running state.
///
/// # Arguments
///
/// * `state` — The five running chaining variables.
/// * `block` — Exactly 64 bytes. Shorter slices are zero-extended rather than
///   indexed out of range, so a caller mistake cannot panic.
///
/// # Returns
///
/// The updated chaining variables.
///
/// # Panics
///
/// Never. Every index into `w` is a compile-time constant offset below 80, and
/// the message words are read through `chunks_exact(4).take(16)`, which yields
/// only in-range slices regardless of `block`'s length.
pub(super) fn compress(state: [u32; 5], block: &[u8]) -> [u32; 5] {
    let mut w = [0u32; 80];
    for (i, chunk) in block.chunks_exact(4).take(16).enumerate() {
        w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = state;
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
    let mixed = [a, b, c, d, e];
    let mut out = [0u32; 5];
    for (slot, (prev, next)) in out.iter_mut().zip(state.iter().zip(mixed.iter())) {
        *slot = prev.wrapping_add(*next);
    }
    out
}

/// The per-quarter mixing function and round constant.
///
/// # Arguments
///
/// * `i` — Round index in `0..80`.
/// * `b`, `c`, `d` — Current working variables.
///
/// # Returns
///
/// The `(f, k)` pair for this round. Indices at or above 60 fall in the final
/// arm, so no round index is unhandled and no `unreachable!()` is needed.
fn round(i: usize, b: u32, c: u32, d: u32) -> (u32, u32) {
    match i {
        0..=19 => ((b & c) | (!b & d), 0x5a82_7999),
        20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
        40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
        _ => (b ^ c ^ d, 0xca62_c1d6),
    }
}
