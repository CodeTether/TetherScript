//! Integration coverage for `tetherscript::hash::sha512` and
//! `tetherscript::hash::sha384`.
//!
//! Provenance of every expected value:
//!
//! * The empty string, `"abc"`, the 112-byte two-block string, and the one
//!   million `'a'` digests are the published FIPS 180-4 Appendix D examples
//!   (§D.1–§D.4) and the matching NIST example set for SHA-512/SHA-384.
//! * The `'a'`-repetition digests at lengths 111/112/113/128/240 are not in any
//!   standard; they were computed with an independent reference implementation
//!   and are pinned here solely to lock the 112-mod-128 padding rollover.
//!
//! No value below is a placeholder.

use tetherscript::hash::sha384::{sha384, sha384_hex};
use tetherscript::hash::sha512::{sha512, sha512_hex};

/// FIPS 180-4 §D.2/§D.4 multi-block example: 112 bytes, which is *exactly* the
/// SHA-512 block boundary (112 mod 128). The 16-byte length field cannot follow
/// the `0x80` byte in the same block, so this must produce two blocks.
const BOUNDARY_112: &[u8] = b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";

/// `'a'` repeated `n` times.
fn a(n: usize) -> Vec<u8> {
    vec![b'a'; n]
}

/// Lowercase hex, defined locally so this file does not depend on a
/// crate-private helper.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn fips_180_4_sha512_vectors() {
    assert_eq!(
        sha512_hex(b""),
        concat!(
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce",
            "47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        )
    );
    assert_eq!(
        sha512_hex(b"abc"),
        concat!(
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a",
            "2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        )
    );
    assert_eq!(
        sha512_hex(BOUNDARY_112),
        concat!(
            "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018",
            "501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909",
        )
    );
}

#[test]
fn fips_180_4_sha384_vectors() {
    assert_eq!(
        sha384_hex(b""),
        concat!(
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da",
            "274edebfe76f65fbd51ad2f14898b95b",
        )
    );
    assert_eq!(
        sha384_hex(b"abc"),
        concat!(
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed",
            "8086072ba1e7cc2358baeca134c825a7",
        )
    );
    assert_eq!(
        sha384_hex(BOUNDARY_112),
        concat!(
            "09330c33f71147e83d192fc782cd1b4753111b173b3b05d22fa08086e3b0f712",
            "fcc7c71a557e2db966c3e9fa91746039",
        )
    );
}

#[test]
fn million_a_vectors() {
    assert_eq!(
        sha512_hex(&a(1_000_000)),
        concat!(
            "e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973eb",
            "de0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b",
        )
    );
    assert_eq!(
        sha384_hex(&a(1_000_000)),
        concat!(
            "9d0e1809716474cb086e834e310a4a1ced149e9c00f248527972cec5704c2a5b",
            "07b8b3dc38ecc4ebae97ddd87f3d8985",
        )
    );
}

#[test]
fn sha384_is_not_truncated_sha512() {
    // SHA-384 starts from a different IV (square roots of primes 23..53), so its
    // digest is not the SHA-512 digest cut short. Anyone "simplifying" sha384
    // into &sha512(..)[..48] breaks this test on the first message.
    let truncated = hex(&sha512(b"abc")[..48]);
    assert_eq!(truncated.len(), 96);
    assert_eq!(truncated.as_str(), &sha512_hex(b"abc")[..96]);
    assert_ne!(sha384_hex(b"abc"), truncated);
    let long = a(200);
    let messages: [&[u8]; 4] = [b"", b"abc", BOUNDARY_112, long.as_slice()];
    for message in messages {
        assert_ne!(sha384(message).as_slice(), &sha512(message)[..48]);
    }
}

#[test]
fn boundary_lengths_need_an_extra_block() {
    // 112 and 240 are both == 112 mod 128, one and two blocks in.
    assert_eq!(BOUNDARY_112.len() % 128, 112);
    assert_eq!(
        sha512_hex(&a(112)),
        concat!(
            "c01d080efd492776a1c43bd23dd99d0a2e626d481e16782e75d54c2503b5dc32",
            "bd05f0f1ba33e568b88fd2d970929b719ecbb152f58f130a407c8830604b70ca",
        )
    );
    assert_eq!(
        sha512_hex(&a(240)),
        concat!(
            "4c296d90c61052a62ffb1dd196f1b7b09373b1f93e71836baebf89690546b759",
            "5684dbe9467a8e484fa0d1094272b4344a7c24f5fee8daedeb0bf549c985ab5f",
        )
    );
    assert_eq!(
        sha384_hex(&a(112)),
        concat!(
            "187d4e07cb306103c69967bf544d0dfbe9042577599c73c330abc0cb64c61236",
            "d5ed565ee19119d8c31779a38f791fcd",
        )
    );
    assert_eq!(
        sha384_hex(&a(240)),
        concat!(
            "4d86957beab348a29180f02d02564ac1d32f5b4c217ece2b038f7c184f0cafc8",
            "c8e438eb82aa03796170e0a7ce8c0675",
        )
    );
}

#[test]
fn lengths_adjacent_to_the_boundary_differ() {
    // 111 still has room for the 16-byte length field; 113 does not.
    assert_eq!(
        sha512_hex(&a(111)),
        concat!(
            "fa9121c7b32b9e01733d034cfc78cbf67f926c7ed83e82200ef8681819692176",
            "0b4beff48404df811b953828274461673c68d04e297b0eb7b2b4d60fc6b566a2",
        )
    );
    assert_eq!(
        sha512_hex(&a(113)),
        concat!(
            "55ddd8ac210a6e18ba1ee055af84c966e0dbff091c43580ae1be703bdb85da31",
            "acf6948cf5bd90c55a20e5450f22fb89bd8d0085e39f85a86cc46abbca75e24d",
        )
    );
    assert_ne!(sha512_hex(&a(111)), sha512_hex(&a(112)));
    assert_ne!(sha512_hex(&a(112)), sha512_hex(&a(113)));
}

#[test]
fn exact_block_multiple() {
    // 128 bytes: a full block with no room for padding at all.
    assert_eq!(
        sha512_hex(&a(128)),
        concat!(
            "b73d1929aa615934e61a871596b3f3b33359f42b8175602e89f7e06e5f658a24",
            "3667807ed300314b95cacdd579f3e33abdfbe351909519a846d465c59582f321",
        )
    );
}

#[test]
fn every_length_under_200_is_distinct_and_panic_free() {
    let mut seen512 = std::collections::HashSet::new();
    let mut seen384 = std::collections::HashSet::new();
    for len in 0..200usize {
        let message = a(len);
        assert_eq!(sha512(&message).len(), 64);
        assert_eq!(sha384(&message).len(), 48);
        assert!(seen512.insert(sha512(&message)), "sha512 repeat at {len}");
        assert!(seen384.insert(sha384(&message)), "sha384 repeat at {len}");
    }
    assert_eq!(seen512.len(), 200);
    assert_eq!(seen384.len(), 200);
}

#[test]
fn hex_widths_and_case() {
    assert_eq!(sha512_hex(b"abc").len(), 128);
    assert_eq!(sha384_hex(b"abc").len(), 96);
    assert!(sha512_hex(b"abc").chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(sha384_hex(b"abc"), sha384_hex(b"abc").to_ascii_lowercase());
}
