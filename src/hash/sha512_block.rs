//! Single-block SHA-512 compression function (FIPS 180-4 §6.4.2).
//!
//! 64-bit words, 80 rounds, 128-byte blocks. The rotation amounts differ from
//! SHA-256 and are not derivable from it, so they are written out literally:
//! `Sigma0 = rotr 28,34,39`, `Sigma1 = rotr 14,18,41`,
//! `sigma0 = rotr1 ^ rotr8 ^ shr7`, `sigma1 = rotr19 ^ rotr61 ^ shr6`.

use crate::hash::sha512_consts::K;

/// Expand a 128-byte chunk into the 80-word SHA-512 message schedule.
fn schedule(chunk: &[u8]) -> [u64; 80] {
    let mut w = [0u64; 80];
    for (slot, word) in w.iter_mut().take(16).zip(chunk.chunks_exact(8)) {
        *slot = u64::from_be_bytes(word.try_into().expect("8-byte word"));
    }
    for i in 16..80 {
        let a = w[i - 15];
        let b = w[i - 2];
        let s0 = a.rotate_right(1) ^ a.rotate_right(8) ^ (a >> 7);
        let s1 = b.rotate_right(19) ^ b.rotate_right(61) ^ (b >> 6);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    w
}

/// Mix one 128-byte chunk into the eight-word state `h`.
///
/// # Arguments
///
/// * `h` — Running state, updated in place.
/// * `chunk` — Exactly 128 message bytes.
pub(crate) fn compress(h: &mut [u64; 8], chunk: &[u8]) {
    let w = schedule(chunk);
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for (i, word) in w.iter().enumerate() {
        let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
        let ch = (e & f) ^ ((!e) & g);
        let t1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(*word);
        let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
        let t2 = s0.wrapping_add((a & b) ^ (a & c) ^ (b & c));
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *slot = slot.wrapping_add(value);
    }
}
