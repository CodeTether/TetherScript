//! Integration coverage for `tetherscript::hash::sha1`.
//!
//! Provenance of every expected value here:
//!
//! * The empty string, `"abc"`, the 56-byte alphabet string, and the one-million
//!   `'a'` digests are the four published vectors in RFC 3174 §7.3 (identical to
//!   FIPS 180-4 §D and to the NIST example set).
//! * The `'a'`-repetition digests at lengths 55/56/57/64/120/128/184 are not in
//!   any standard, so they were computed with an independent reference SHA-1 and
//!   are pinned here. They exist to lock down the padding rollover, which the
//!   RFC vectors only cover at length 56.
//!
//! No value below is a placeholder.

use tetherscript::hash::sha1::{sha1, sha1_hex};

/// RFC 3174 §7.3 test vector 3: 56 bytes, which is *exactly* the SHA-1 block
/// boundary. After the mandatory `0x80` byte the 8-byte length field no longer
/// fits in the first block, so a correct implementation emits two blocks. An
/// implementation that computes padding before appending `0x80` gets this wrong
/// while still passing `"abc"`.
const BOUNDARY_56: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

/// `'a'` repeated `n` times.
fn a(n: usize) -> Vec<u8> {
    vec![b'a'; n]
}

#[test]
fn rfc3174_published_vectors() {
    assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert_eq!(
        sha1_hex(BOUNDARY_56),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}

#[test]
fn million_a_vector() {
    // RFC 3174 §7.3 test vector 4.
    assert_eq!(
        sha1_hex(&a(1_000_000)),
        "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
    );
}

#[test]
fn boundary_lengths_need_an_extra_block() {
    // 56, 120 and 184 are all == 56 mod 64: one, two and three blocks in.
    assert_eq!(BOUNDARY_56.len() % 64, 56);
    assert_eq!(sha1_hex(&a(56)), "c2db330f6083854c99d4b5bfb6e8f29f201be699");
    assert_eq!(
        sha1_hex(&a(120)),
        "f34c1488385346a55709ba056ddd08280dd4c6d6"
    );
    assert_eq!(
        sha1_hex(&a(184)),
        "2a4c545628c4875631e342e101f8af11cf48d252"
    );
}

#[test]
fn lengths_adjacent_to_the_boundary_differ() {
    // 55 still has room for its length field; 57 does not.
    assert_eq!(sha1_hex(&a(55)), "c1c8bbdc22796e28c0e15163d20899b65621d65a");
    assert_eq!(sha1_hex(&a(57)), "f08f24908d682555111be7ff6f004e78283d989a");
    assert_ne!(sha1_hex(&a(55)), sha1_hex(&a(56)));
    assert_ne!(sha1_hex(&a(56)), sha1_hex(&a(57)));
}

#[test]
fn exact_block_multiples() {
    // A full block with no room at all for padding.
    assert_eq!(sha1_hex(&a(64)), "0098ba824b5c16427bd7a1122a5a442a25ec644d");
    assert_eq!(
        sha1_hex(&a(128)),
        "ad5b3fdbcb526778c2839d2f151ea753995e26a0"
    );
}

#[test]
fn every_length_under_200_is_distinct_and_panic_free() {
    // Covers every padding path across three block rollovers.
    let mut seen = std::collections::HashSet::new();
    for len in 0..200usize {
        let digest = sha1(&a(len));
        assert_eq!(digest.len(), 20);
        assert!(seen.insert(digest), "digest repeated at length {len}");
    }
    assert_eq!(seen.len(), 200);
}

#[test]
fn hex_is_lowercase_and_forty_chars() {
    let hex = sha1_hex(b"abc");
    assert_eq!(hex.len(), 40);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(hex, hex.to_ascii_lowercase());
}
