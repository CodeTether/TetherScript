//! The bucket function: `(seed, subject)` to an integer in `0..10000`.
//!
//! # Why a hash and not a random number generator
//!
//! Assignment must be *stable*. A visitor who is shown the control on one request
//! and the treatment on the next both invalidates the experiment and looks broken.
//! A hash is a pure function of its input, so the same `(seed, subject)` yields the
//! same bucket in every process, on every host, after every restart, with no state
//! stored anywhere. An RNG cannot promise that, so none is used here.
//!
//! The digest is the in-tree [`crate::system::sha256`], reused rather than
//! reimplemented. SHA-256 is not needed for its collision resistance here — it is
//! needed because its output bits are uniformly distributed and it is already
//! present with zero dependencies.
//!
//! # Avoiding modulo bias
//!
//! Two independent mistakes are avoided.
//!
//! *Too few bytes.* Folding a 2-byte digest prefix with `% 10000` would be badly
//! biased: a `u16` spans 65536 values and `65536 % 10000 == 5536`, so buckets
//! `0..5536` would each be reachable from 7 hash values while `5536..10000` would
//! be reachable from only 6 — a systematic ~16% over-weighting of the low buckets,
//! which is exactly the range a small first variant occupies. Eight bytes are taken
//! instead, giving a `u64` domain of 2^64 values.
//!
//! *Modulo at all.* Even over `u64` a `%` leaves a residue of `2^64 % 10000`
//! values, so the low buckets remain very slightly favoured. The relative excess is
//! under `10000 / 2^64`, about 5e-16, which is unmeasurable — but the residue is
//! removed entirely by scaling instead of dividing: the 64-bit value is widened to
//! `u128`, multiplied by 10000, and shifted right by 64. That is a fixed-point
//! multiply by `10000 / 2^64`, so each bucket claims a contiguous slice of the
//! 64-bit domain and the slices differ in size by at most one value.
//!
//! Which eight bytes is not arbitrary either: the *leading* eight are used, and the
//! seed is hashed as a prefix rather than concatenated raw, so no seed/subject pair
//! can be spelled two ways. See [`hash_input`].

use crate::system::sha256;

/// Number of buckets. Basis points, so a 0.01% traffic slice is still expressible.
pub(super) const BUCKETS: u128 = 10_000;

/// Separator between seed and subject.
///
/// Without it, `("ab", "cd")` and `("a", "bcd")` would hash identically, and two
/// unrelated experiments could share an assignment purely by how their names
/// happened to split. `0x1f` (ASCII unit separator) is not a byte that appears in a
/// cookie value, a UUID, or a session id.
const SEP: u8 = 0x1f;

/// Compute the bucket for a subject under a seed.
///
/// # Arguments
///
/// * `seed` — Experiment seed. Changing it reshuffles every subject, which is how
///   a fresh experiment avoids inheriting the previous one's population split.
/// * `subject` — Stable subject identifier: a visitor id or a session id.
///
/// # Returns
///
/// An integer in `0..10000`, uniformly distributed over subjects. Never negative
/// and never 10000, so it always indexes a half-open weight range.
pub(super) fn bucket(seed: &str, subject: &str) -> i64 {
    let digest = sha256(&hash_input(seed, subject));
    let mut leading = [0u8; 8];
    leading.copy_from_slice(&digest[..8]);
    let value = u64::from_be_bytes(leading) as u128;
    ((value * BUCKETS) >> 64) as i64
}

/// Build the unambiguous digest input for a `(seed, subject)` pair.
///
/// The separator makes the encoding injective, so distinct pairs always produce
/// distinct input bytes.
fn hash_input(seed: &str, subject: &str) -> Vec<u8> {
    let mut input = Vec::with_capacity(seed.len() + subject.len() + 1);
    input.extend_from_slice(seed.as_bytes());
    input.push(SEP);
    input.extend_from_slice(subject.as_bytes());
    input
}
