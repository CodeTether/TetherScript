//! Knuth Algorithm D: multi-limb long division (TAOCP vol. 2, §4.3.1).
//!
//! Steps, in the book's numbering:
//!
//! * **D1 normalize** — shift both operands left so the divisor's top limb has
//!   its high bit set. This is what bounds the quotient-digit estimate error at
//!   one, and it is the step naive implementations skip.
//! * **D2/D7 loop** — walk quotient positions from most to least significant.
//! * **D3 estimate** — a two-limb `qhat` guess (see `div_estimate`).
//! * **D4/D5/D6 multiply, subtract, add back** — see `div_mulsub`.
//! * **D8 unnormalize** — shift the remainder back right by the same amount.
//!
//! Requires a divisor of at least two limbs and a dividend at least as large;
//! `Uint::divmod` guarantees both before calling.

use super::div_estimate::{estimate, shift_fixed};
use super::div_mulsub::{add_back, mul_sub};
use super::uint::Uint;

/// Divides `u` by `v`, returning `(quotient, remainder)`.
///
/// # Panics
///
/// Panics (via `debug_assert!`) if the caller violates the preconditions:
/// `v` must have at least two limbs and `u >= v`.
pub(super) fn divmod_knuth(u: &Uint, v: &Uint) -> (Uint, Uint) {
    let n = v.limbs.len();
    debug_assert!(n >= 2, "Algorithm D needs a two-limb divisor");
    debug_assert!(u.limbs.len() >= n, "caller must ensure u >= v");
    let m = u.limbs.len() - n;

    let shift = v.limbs[n - 1].leading_zeros() as usize;
    let vn = shift_fixed(&v.limbs, shift, n);
    let mut un = shift_fixed(&u.limbs, shift, m + n + 1);

    let mut q = vec![0u64; m + 1];
    for j in (0..=m).rev() {
        let mut qhat = estimate(&un[j..=j + n], &vn);
        if mul_sub(&mut un[j..=j + n], &vn, qhat) {
            // qhat was one too large: undo the over-subtraction.
            qhat -= 1;
            add_back(&mut un[j..=j + n], &vn);
        }
        q[j] = qhat;
    }

    let rem = Uint::from_limbs(un[..n].to_vec()).shr(shift);
    (Uint::from_limbs(q), rem)
}
