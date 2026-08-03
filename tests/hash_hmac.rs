//! Integration coverage for `tetherscript::hash::hmac_sha1` and
//! `tetherscript::hash::hmac_sha512`.
//!
//! Provenance:
//!
//! * HMAC-SHA-1 vectors are RFC 2202 §3, test cases 1–7, quoted verbatim.
//! * HMAC-SHA-512 vectors are RFC 4231 §4, test cases 1–7, quoted verbatim.
//! * The two extra 65-byte-key checks are not from a standard. They exist to
//!   prove the block size is 128 for SHA-512 and 64 for SHA-1: a 65-byte key is
//!   *over* SHA-1's block (so it is hashed first) and *under* SHA-512's (so it
//!   is zero-padded). Their expected values were computed with an independent
//!   reference HMAC.
//!
//! No value below is a placeholder.

use tetherscript::hash::hmac_sha1::{hmac_sha1, hmac_sha1_hex};
use tetherscript::hash::hmac_sha512::{hmac_sha512, hmac_sha512_hex};

/// `byte` repeated `n` times.
fn rep(byte: u8, n: usize) -> Vec<u8> {
    vec![byte; n]
}

#[test]
fn rfc2202_sha1_cases_1_to_5() {
    // Case 1: key = 20 x 0x0b, data = "Hi There".
    assert_eq!(
        hmac_sha1_hex(&rep(0x0b, 20), b"Hi There"),
        "b617318655057264e28bc0b6fb378c8ef146be00"
    );
    // Case 2: key = "Jefe".
    assert_eq!(
        hmac_sha1_hex(b"Jefe", b"what do ya want for nothing?"),
        "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"
    );
    // Case 3: key = 20 x 0xaa, data = 50 x 0xdd.
    assert_eq!(
        hmac_sha1_hex(&rep(0xaa, 20), &rep(0xdd, 50)),
        "125d7342b9ac11cd91a39af48aa17b4f63f175d3"
    );
    // Case 4: key = 0x0102...19, data = 50 x 0xcd.
    let key: Vec<u8> = (1u8..=25).collect();
    assert_eq!(
        hmac_sha1_hex(&key, &rep(0xcd, 50)),
        "4c9007f4026250c6bc8414f9bf50c86c2d7235da"
    );
    // Case 5: key = 20 x 0x0c; RFC 2202 also publishes the 96-bit truncation.
    let mac = hmac_sha1_hex(&rep(0x0c, 20), b"Test With Truncation");
    assert_eq!(mac, "4c1a03424b55e07fe7f27be1d58bb9324a9a5a04");
    assert_eq!(&mac[..24], "4c1a03424b55e07fe7f27be1");
}

#[test]
fn rfc2202_sha1_oversized_keys_are_hashed_first() {
    // Cases 6 and 7 use an 80-byte key, longer than SHA-1's 64-byte block, so
    // RFC 2104 requires hashing the key down to 20 bytes before padding.
    let key = rep(0xaa, 80);
    assert_eq!(
        hmac_sha1_hex(
            &key,
            b"Test Using Larger Than Block-Size Key - Hash Key First"
        ),
        "aa4ae5e15272d00e95705637ce8a3b55ed402112"
    );
    assert_eq!(
        hmac_sha1_hex(
            &key,
            b"Test Using Larger Than Block-Size Key and Larger Than One Block-Size Data"
        ),
        "e8e99d0f45237d786d6bbaa7965c7808bbff1a91"
    );
}

#[test]
fn rfc4231_sha512_cases_1_to_5() {
    // Case 1.
    assert_eq!(
        hmac_sha512_hex(&rep(0x0b, 20), b"Hi There"),
        concat!(
            "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cde",
            "daa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854",
        )
    );
    // Case 2.
    assert_eq!(
        hmac_sha512_hex(b"Jefe", b"what do ya want for nothing?"),
        concat!(
            "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea250554",
            "9758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737",
        )
    );
    // Case 3.
    assert_eq!(
        hmac_sha512_hex(&rep(0xaa, 20), &rep(0xdd, 50)),
        concat!(
            "fa73b0089d56a284efb0f0756c890be9b1b5dbdd8ee81a3655f83e33b2279d39",
            "bf3e848279a722c806b485a47e67c807b946a337bee8942674278859e13292fb",
        )
    );
    // Case 4.
    let key: Vec<u8> = (1u8..=25).collect();
    assert_eq!(
        hmac_sha512_hex(&key, &rep(0xcd, 50)),
        concat!(
            "b0ba465637458c6990e5a8c5f61d4af7e576d97ff94b872de76f8050361ee3db",
            "a91ca5c11aa25eb4d679275cc5788063a5f19741120c4f2de2adebeb10a298dd",
        )
    );
    // Case 5 publishes only the leading 128 bits.
    let mac = hmac_sha512_hex(&rep(0x0c, 20), b"Test With Truncation");
    assert_eq!(&mac[..32], "415fad6271580a531d4179bc891d87a6");
}

#[test]
fn rfc4231_sha512_oversized_keys_are_hashed_first() {
    // Cases 6 and 7 use a 131-byte key, longer than SHA-512's 128-byte block.
    // A 64-byte BLOCK constant would hash this key too — coincidentally still
    // "working" — but would break the 65..128-byte range covered below.
    let key = rep(0xaa, 131);
    assert_eq!(
        hmac_sha512_hex(
            &key,
            b"Test Using Larger Than Block-Size Key - Hash Key First"
        ),
        concat!(
            "80b24263c7c1a3ebb71493c1dd7be8b49b46d1f41b4aeec1121b013783f8f352",
            "6b56d037e05f2598bd0fd2215d6a1e5295e64f73f63f0aec8b915a985d786598",
        )
    );
    let long_data: &[u8] = concat!(
        "This is a test using a larger than block-size key and a larger ",
        "than block-size data. The key needs to be hashed before being used ",
        "by the HMAC algorithm.",
    )
    .as_bytes();
    assert_eq!(
        hmac_sha512_hex(&key, long_data),
        concat!(
            "e37b6a775dc87dbaa4dfa9f96e5e3ffddebd71f8867289865df5a32d20cdc944",
            "b6022cac3c4982b10d5eeb55c3e4de15134676fb6de0446065c97440fa8c6a58",
        )
    );
}

#[test]
fn block_size_is_per_hash_not_per_digest() {
    // A 65-byte key straddles the two block sizes: SHA-1 must hash it (65 > 64),
    // SHA-512 must zero-pad it (65 <= 128). If HMAC-SHA-512 used 64 as its block
    // size it would hash this key instead, silently producing a MAC that no
    // other implementation agrees with. These two values are what a conformant
    // implementation returns.
    let key = rep(0xaa, 65);
    assert_eq!(
        hmac_sha1_hex(&key, b"boundary"),
        "0f1250f9d62b88bac1700be7218f574794b0560c"
    );
    assert_eq!(
        hmac_sha512_hex(&key, b"boundary"),
        concat!(
            "cb4c88b0850217ad51c335bee99b6d109436bd752fa9c54c8ec6c835932d2c3a",
            "a91804b0e9238f659ea836c15472d175ea72b291c48dd9f6e4c994b9ac1db056",
        )
    );
}

#[test]
fn sha512_key_at_and_past_its_own_block_boundary_differ() {
    // 128 bytes is padded (no-op); 129 bytes is hashed. The two must differ.
    let at = hmac_sha512_hex(&rep(0xaa, 128), b"boundary");
    let past = hmac_sha512_hex(&rep(0xaa, 129), b"boundary");
    assert_eq!(
        at,
        concat!(
            "fc204fb23809693938980fe07ef2085806225f541df2b6e0a86cc60644ca86de",
            "699120df49ada84b391d627a335620423647e60983f1dcd6b4fc450053e3390c",
        )
    );
    assert_eq!(
        past,
        concat!(
            "1d41e0211baf770c55c3709926ca9f8fa94712d131daedd2d071d147c143e9cf",
            "d19e29bba82b45c2b9af113bd04503e40d287acf0d9e0db7d2617633ccc54acc",
        )
    );
    assert_ne!(at, past);
}

#[test]
fn mac_widths_and_empty_inputs() {
    assert_eq!(hmac_sha1(b"", b"").len(), 20);
    assert_eq!(hmac_sha512(b"", b"").len(), 64);
    assert_eq!(hmac_sha1_hex(b"k", b"m").len(), 40);
    assert_eq!(hmac_sha512_hex(b"k", b"m").len(), 128);
}
