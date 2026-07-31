//! The MD5 block compression function.

use super::md5_constants::{K, SHIFTS};

/// Mix one 64-byte block into the running state.
pub(super) fn compress(state: [u32; 4], m: &[u32; 16]) -> [u32; 4] {
    let [mut a, mut b, mut c, mut d] = state;
    for i in 0..64 {
        let (f, g) = round(i, b, c, d);
        let tmp = d;
        d = c;
        c = b;
        let sum = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]);
        b = b.wrapping_add(sum.rotate_left(SHIFTS[i]));
        a = tmp;
    }
    [
        state[0].wrapping_add(a),
        state[1].wrapping_add(b),
        state[2].wrapping_add(c),
        state[3].wrapping_add(d),
    ]
}

/// Per-quarter mixing function and message-word index.
fn round(i: usize, b: u32, c: u32, d: u32) -> (u32, usize) {
    match i / 16 {
        0 => ((b & c) | (!b & d), i),
        1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
        2 => (b ^ c ^ d, (3 * i + 5) % 16),
        _ => (c ^ (b | !d), (7 * i) % 16),
    }
}
