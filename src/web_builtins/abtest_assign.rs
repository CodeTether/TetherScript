//! Mapping a bucket onto a variant.
//!
//! # How the boundary is computed
//!
//! Weights are percentage points summing to 100 and buckets span `0..10000`, so one
//! percentage point is exactly 100 buckets and no rounding is involved anywhere. A
//! variant owns the half-open range `[cumulative * 100, (cumulative + weight) * 100)`
//! where `cumulative` is the sum of the weights declared before it.
//!
//! Half-open is what makes the boundary predictable: raising the first variant's
//! weight from 50 to 60 moves the boundary from bucket 5000 to bucket 6000 and moves
//! exactly the subjects in `5000..6000`. Every other subject keeps its variant,
//! because the ranges below and above are unchanged. That is the desired property —
//! a weight change should migrate the minimum number of visitors — and it is why the
//! variants are scanned in configured order rather than sorted.
//!
//! A `0` weight yields an empty range, so a `100/0` split sends every subject to the
//! first variant and never selects the second.

use super::abtest_bucket::BUCKETS;
use super::abtest_variant::Variant;
use super::abtest_weight::TOTAL_WEIGHT;

/// Buckets per percentage point. Exact, because 10000 is divisible by 100.
const PER_POINT: i64 = BUCKETS as i64 / TOTAL_WEIGHT;

/// Select the variant owning `bucket`.
///
/// # Arguments
///
/// * `variants` — Validated variants in configured order, weights summing to 100.
/// * `bucket` — A bucket in `0..10000`, from `abtest_bucket::bucket`.
///
/// # Returns
///
/// The name of the variant whose half-open bucket range contains `bucket`.
///
/// # Panics
///
/// Does not panic. The fallback branch cannot be reached for a validated
/// experiment — the ranges tile `0..10000` exactly — but rather than assert that, the
/// last non-zero-weight variant is returned, so a caller that hand-built a map with
/// an unchecked weight sum still gets a real variant instead of a crash mid-request.
pub(super) fn select(variants: &[Variant], bucket: i64) -> String {
    let mut boundary = 0;
    for variant in variants {
        boundary += variant.weight * PER_POINT;
        if bucket < boundary {
            return variant.name.clone();
        }
    }
    variants
        .iter()
        .rev()
        .find(|variant| variant.weight > 0)
        .or_else(|| variants.last())
        .map(|variant| variant.name.clone())
        .unwrap_or_default()
}
