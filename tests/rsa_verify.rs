//! Integration coverage for `tetherscript::rsa::verify` and key admission.
//!
//! # The known-good vector
//!
//! There is no PKCS#1 v1.5 test vector that can be verified without a real
//! 2048-bit key, so one was generated and its correctness cross-checked two
//! independent ways before being frozen here.
//!
//! Generation (recorded so the vector is reproducible in spirit, not rerun):
//!
//! ```text
//! openssl genrsa -out k.pem 2048
//! printf 'tetherscript rsa pkcs1 v1_5 test vector' > msg.txt
//! openssl dgst -sha256 -sign k.pem -out sig.bin msg.txt
//! openssl rsa -in k.pem -noout -modulus      # -> N_HEX below, e = 0x010001
//! ```
//!
//! Cross-check 1 — the signer: OpenSSL produced `sig.bin`, so the vector is a
//! genuine RSASSA-PKCS1-v1_5 signature, not one this code shaped for itself.
//!
//! Cross-check 2 — the recovered block, computed independently of this crate:
//! `pow(OS2IP(sig), 65537, N)` rendered as 256 octets is
//!
//! ```text
//! 0001 ff*202 00 3031300d060960864801650304020105000420
//!      72a693f323b6c6b310e45a4d28e809c6344daf2203b7effd5b088840aafbcc3f
//! ```
//!
//! The 202 padding octets satisfy `k - tLen - 3 = 256 - 51 - 3 = 202`, the DER
//! prefix is exactly the RFC 8017 section 9.2 SHA-256 `DigestInfo` prefix, and
//! the trailing 32 octets are `SHA-256("tetherscript rsa pkcs1 v1_5 test
//! vector")`, which is `72a693f3...aafbcc3f`. So the block is byte-for-byte what
//! this implementation must accept, derived without running it.
//!
//! The same message was also signed under SHA-1, SHA-384, and SHA-512 with the
//! same key, giving four vectors that additionally pin the per-algorithm
//! `DigestInfo` prefixes against a real signer.
//!
//! # Hex, not byte arrays
//!
//! Vectors are stored as hex and decoded through `BigUint::from_hex` plus
//! `to_be_bytes(width)`, so no ad-hoc hex decoder is introduced and the fixed
//! octet width is stated explicitly at every use.

use tetherscript::bigint::BigUint;
use tetherscript::rsa::{verify, DigestAlgorithm, RsaError, RsaPublicKey};

/// Modulus `n` of the 2048-bit test key, big-endian hex (512 digits).
const N_HEX: &str = "a3203fde9f4f871d76efdb25121255487de28880c3c8dc6d136a3faf2cf6810b03d236e9f088c91097ab0a6a4493b9aa53852691d6e61f075861c479f7ee5404be6d4d6736befd5cf254efb42a281d77e5c80c7c931e05a47f2c1cabbafc1c2e1ca9798658cf262a145223b8397d121e59af6bcdd3afcd6e3f894d708e8919646c9edeb36b4b260ecf930e6f9dcf7a8a98d2811ed2270f85883eebc1a4113cfdcca25000fbb885c60353a4f06818b180d80f38a188c40ae33a53cf3b70ebe38acdae3471ab5df22817eec19de3135e5633ef53701e47d5b5a78b984496c18b8865d64845bcc78bc0751e5594dd7f3eb63dda27a194d195cf947afaed766fa405";

/// SHA-256 signature over the test message.
const SIG_SHA256: &str = "94fc855487043a4379f654efecc862d5b4881dd4b85194cd8f466e0ad10ff047ffe035707eb477a542dd320205e0a8640b43a6c16e80a52b3c8e0038561c1d0a5b6419dc5cc4cd0af30e972b01f72b7549d24ad8eb4f2f607a53b4f8f6e4fca0dd639216138cb6aa77d249dcf361e58d52a06ce164d2bb8bf723f4bc5432e86ff879a0284cec9f2b7205c46b660c0543a6582fd2c3b3a5f985673d97e7c1e6a22c0e007cebf6c167d19e76681bc89a508c6706754dbdc4b86b691462ac86bfe0ae985a5d14b24476a6f5ad44b5c198bc836b91bef264af217c744be427c0e676f9936340930551a7cc32d3225d0fa6b57ced4bb5bbc9e0a619b17152f294bd64";

/// `SHA-256("tetherscript rsa pkcs1 v1_5 test vector")`.
const DIGEST_SHA256: &str = "72a693f323b6c6b310e45a4d28e809c6344daf2203b7effd5b088840aafbcc3f";

/// SHA-1 signature over the same message under the same key.
const SIG_SHA1: &str = "94141baeff0b600ffa03c71c2fc71b13418b128536f9ed0b1fd6035c9a9b7031d5ca4a83a43819f9b33b94071b9746087bd47c8adb20e78505691124b53419640ec2e4b9ce49936565267f6d9191cd64e85037d2696e16d7c8c1fa76dae7d2da99a8217034c02090e910224206c0d78e19d88892506c85627873477b1c19a2a42482e11f46a23da2e95f2acc8ed12cffee82ec53e8431736a7c835fe7d6ccabe2adf69827397301e36d61a59200d20870d38e0514c4fa7b0f4b063310998c03d6f77d9ac9a53356049d4b01c3e70483d2ee55a2c268f278e122b31726b098ec0fc18df127033706460e03856a880bcf0dc01f04c7918e26c211767a632d6aed7";

/// `SHA-1("tetherscript rsa pkcs1 v1_5 test vector")`.
const DIGEST_SHA1: &str = "4f5fb4ed795dac47aae1ca4719ebc9492807f5b3";

/// SHA-384 signature over the same message under the same key.
const SIG_SHA384: &str = "14cdc4b27353b00c8434f360864d5b171d25e1e7334afcde4b99382abb93a1ff701375caf29de799d4c9472b5cc0f3a3f1ba017e6c56c7b3f2540f614f165590a520acdb38fa20ac106cb220f7ab8dfe6e92966c445ffcd8a2aed41cbfa4534361d7ef1aaf7aa023ee36e0872b64f467bf9a4f6ee68544148877232eaeabaef3cc2467ce45309f15e82cd566818ef282d00e4d5d9e949416dfa181f35f702876656c577294fbf0c3b8718ac2f07c9f09a36b788c8799aeb8aea27e7440e80325acd73e22ec4f6934193c459a1571f9c3e9859286ac2698a180957ec973e0aa21fcb4868c2cb404054b3461361a852ae4ed8b1b9c045232a1063d8d0cef620ba1";

/// `SHA-384("tetherscript rsa pkcs1 v1_5 test vector")`.
const DIGEST_SHA384: &str = "030ace9c95c80f4ecb9ff84134a2a2900ac872cb09b3c8fd296011478f1fd4630c89543c788a6cf573b1434637be3b4c";

/// SHA-512 signature over the same message under the same key.
const SIG_SHA512: &str = "62956abcfbecb62c95e6f92f8274d082c4805f92763e2dce0ac945e0ac396d2695b97dec7bbc4767d16010f82c35385d1adfe2f02390e07b5a0b1703678bcd8c5a6bfd3d3605a935bc84be41df0215fc1b0420677b57275b0e215f0e6002f62b63038489362f5b5f6761308f4c8a1aa34b6f97d431c107c240c6c021059166538d79c21a3bb1f938c8a1568a920d2c8724d37175fa7d99bc884ae7cc128d463d630769ef3040047a99fa9ed6eaac26f24f99d7742bc3b5ea12ca1dadb9698672fdaae7032f91403f47d0fc7deeb78b4284c310c82d725277769070b6358d34f672c3438e8bc69196402147ff73f802aeb5a4084939030f22d7cdedced3079259";

/// `SHA-512("tetherscript rsa pkcs1 v1_5 test vector")`.
const DIGEST_SHA512: &str = "393c8754a1413e0958d640054e304c5368bdc5678f4615079831f506274a0517e864ebe17f395e49d2541c56ccfdf1f43d4d918220e21d058ac22faea720864f";

/// Decode a big-endian hex vector to exactly `width` octets.
fn octets(hex: &str, width: usize) -> Vec<u8> {
    BigUint::from_hex(hex)
        .expect("test vector must be valid hex")
        .to_be_bytes(width)
        .expect("test vector must fit the stated width")
}

/// The 2048-bit test public key with `e = 65537`.
fn key() -> RsaPublicKey {
    RsaPublicKey::new(
        BigUint::from_hex(N_HEX).expect("modulus hex"),
        BigUint::from_u64(65_537),
    )
    .expect("the test key is 2048-bit, odd, and has e = 65537")
}

#[test]
fn the_test_key_is_exactly_2048_bits_and_odd() {
    let key = key();
    assert_eq!(key.modulus_bits(), 2048);
    assert_eq!(key.modulus_bytes(), 256);
    assert_eq!(key.exponent(), &BigUint::from_u64(65_537));
}

#[test]
fn known_good_sha256_signature_verifies() {
    assert_eq!(
        verify(
            &octets(SIG_SHA256, 256),
            &octets(DIGEST_SHA256, 32),
            DigestAlgorithm::Sha256,
            &key(),
        ),
        Ok(())
    );
}

#[test]
fn known_good_signatures_verify_for_every_algorithm() {
    // Each vector pins one DigestInfo prefix against a real OpenSSL signer.
    let cases = [
        (SIG_SHA1, DIGEST_SHA1, 20, DigestAlgorithm::Sha1),
        (SIG_SHA256, DIGEST_SHA256, 32, DigestAlgorithm::Sha256),
        (SIG_SHA384, DIGEST_SHA384, 48, DigestAlgorithm::Sha384),
        (SIG_SHA512, DIGEST_SHA512, 64, DigestAlgorithm::Sha512),
    ];
    for (sig, digest, width, alg) in cases {
        assert_eq!(
            verify(&octets(sig, 256), &octets(digest, width), alg, &key()),
            Ok(()),
            "{alg:?} vector should verify"
        );
    }
}

#[test]
fn a_valid_signature_is_refused_under_a_different_claimed_algorithm() {
    // The SHA-256 signature's recovered block has a 51-octet DigestInfo region,
    // because PS was sized as k - 51 - 3 = 202. Claiming SHA-512 asks for an
    // 83-octet region, so the refusal lands on the region length and the digest
    // is never compared. Either way the caller's claimed algorithm, not the
    // block's contents, decides what is acceptable.
    let err = verify(
        &octets(SIG_SHA256, 256),
        &octets(DIGEST_SHA512, 64),
        DigestAlgorithm::Sha512,
        &key(),
    )
    .unwrap_err();
    assert_eq!(err, RsaError::DigestInfoLength { expected: 83, found: 51 });
}

#[test]
fn a_sha512_signature_is_refused_under_a_sha256_claim() {
    // The mirror direction: the SHA-512 block's region is 83 octets, so a
    // SHA-256 claim wants 51 and is refused.
    let err = verify(
        &octets(SIG_SHA512, 256),
        &octets(DIGEST_SHA256, 32),
        DigestAlgorithm::Sha256,
        &key(),
    )
    .unwrap_err();
    assert_eq!(err, RsaError::DigestInfoLength { expected: 51, found: 83 });
}

#[test]
fn a_sha384_signature_is_refused_under_a_sha512_claim() {
    // SHA-384 and SHA-512 DigestInfos differ in length too (67 vs 83), so this
    // is again a length refusal; the prefix-identity refusal is exercised on
    // hand-built equal-length blocks in `tests/rsa_pkcs1.rs`.
    let err = verify(
        &octets(SIG_SHA384, 256),
        &octets(DIGEST_SHA512, 64),
        DigestAlgorithm::Sha512,
        &key(),
    )
    .unwrap_err();
    assert_eq!(err, RsaError::DigestInfoLength { expected: 83, found: 67 });
}

#[test]
fn a_flipped_final_digest_octet_is_refused() {
    let mut digest = octets(DIGEST_SHA256, 32);
    *digest.last_mut().unwrap() ^= 0x01;
    assert_eq!(
        verify(&octets(SIG_SHA256, 256), &digest, DigestAlgorithm::Sha256, &key()),
        Err(RsaError::DigestMismatch)
    );
}

#[test]
fn a_flipped_signature_octet_destroys_the_padding() {
    // Changing one signature octet changes the recovered block essentially at
    // random, so the failure lands on a padding rule rather than the digest.
    let mut sig = octets(SIG_SHA256, 256);
    sig[128] ^= 0x01;
    let err = verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key())
        .expect_err("a tampered signature must not verify");
    assert_ne!(err, RsaError::DigestMismatch, "must fail structurally, not on the digest");
}

#[test]
fn a_signature_shorter_than_the_modulus_is_refused() {
    // A 255-octet string, i.e. the vector with its leading octet dropped.
    let sig = octets(SIG_SHA256, 256)[1..].to_vec();
    assert_eq!(
        verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::SignatureLength { got: 255, expected: 256 })
    );
}

#[test]
fn a_signature_longer_than_the_modulus_is_refused() {
    // A leading zero octet does not change the integer, but PKCS#1 fixes the
    // length at k so the representation must be rejected, not normalized.
    let mut sig = vec![0x00];
    sig.extend(octets(SIG_SHA256, 256));
    assert_eq!(
        verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::SignatureLength { got: 257, expected: 256 })
    );
}

#[test]
fn a_signature_equal_to_the_modulus_is_refused() {
    // s == n reduces to 0, so without the range check the recovered block would
    // be all zeros and the failure would be reported as bad padding instead of
    // an out-of-range signature.
    let sig = octets(N_HEX, 256);
    assert_eq!(
        verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::SignatureOutOfRange)
    );
}

#[test]
fn a_signature_greater_than_the_modulus_is_refused() {
    // s = n + 1 reduces to 1. Accepting it would mean every valid signature has
    // a family of accepted aliases s + i*n, so a signature is no longer a unique
    // token and replay caches keyed on its bytes can be bypassed.
    let over = BigUint::from_hex(N_HEX).unwrap().add(&BigUint::from_u64(1));
    let sig = over.to_be_bytes(256).expect("n + 1 still fits in 256 octets");
    assert_eq!(
        verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::SignatureOutOfRange)
    );
}

#[test]
fn an_all_zero_signature_is_refused() {
    // 0^e mod n == 0, whose encoding is 256 zero octets: leading 0x00 0x00.
    assert_eq!(
        verify(&[0u8; 256], &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::LeadingBytes { first: 0x00, second: 0x00 })
    );
}

#[test]
fn a_signature_of_one_is_refused() {
    // 1^e mod n == 1, i.e. 255 zero octets then 0x01: still not 0x00 0x01 at the
    // front, so the trivial "signature" every attacker can produce is refused.
    let mut sig = vec![0u8; 256];
    sig[255] = 0x01;
    assert_eq!(
        verify(&sig, &octets(DIGEST_SHA256, 32), DigestAlgorithm::Sha256, &key()),
        Err(RsaError::LeadingBytes { first: 0x00, second: 0x00 })
    );
}

#[test]
fn an_even_modulus_is_refused_at_construction() {
    // The test modulus ends in 0x05; clearing the low bit makes it even, so it
    // cannot be a product of odd primes.
    let even = BigUint::from_hex(N_HEX).unwrap().sub(&BigUint::from_u64(1)).unwrap();
    assert!(!even.bit(0), "n - 1 must be even for this test to mean anything");
    assert_eq!(
        RsaPublicKey::new(even, BigUint::from_u64(65_537)).unwrap_err(),
        RsaError::ModulusEven
    );
}

#[test]
fn an_exponent_of_one_is_refused_at_construction() {
    // s^1 mod n == s, so the "signature" is the encoded message in the clear and
    // anyone can write it down without the private key.
    assert_eq!(
        RsaPublicKey::new(BigUint::from_hex(N_HEX).unwrap(), BigUint::from_u64(1)).unwrap_err(),
        RsaError::ExponentTooSmall
    );
}

#[test]
fn an_exponent_of_zero_is_refused_at_construction() {
    // s^0 mod n == 1 for every s, so the recovered block never depends on the
    // signature at all.
    assert_eq!(
        RsaPublicKey::new(BigUint::from_hex(N_HEX).unwrap(), BigUint::zero()).unwrap_err(),
        RsaError::ExponentTooSmall
    );
}

#[test]
fn a_1024_bit_modulus_is_refused_at_construction() {
    // Take the low 128 octets of the test modulus: a 1024-bit odd value. It is a
    // structurally fine RSA-shaped integer and is still refused, because the
    // 2048-bit floor is a policy check, not a well-formedness check.
    let low = octets(N_HEX, 256)[128..].to_vec();
    assert_eq!(low.len(), 128);
    let err = RsaPublicKey::from_be_bytes(&low, &[0x01, 0x00, 0x01]).unwrap_err();
    assert_eq!(err, RsaError::ModulusTooSmall { bytes: 128 });
}

#[test]
fn a_modulus_one_octet_under_the_floor_is_refused() {
    // 255 octets is 2040 bits: the boundary must be `< 256`, not `< 255`.
    let mut n = octets(N_HEX, 256)[1..].to_vec();
    n[254] |= 0x01;
    assert_eq!(
        RsaPublicKey::from_be_bytes(&n, &[0x01, 0x00, 0x01]).unwrap_err(),
        RsaError::ModulusTooSmall { bytes: 255 }
    );
}

#[test]
fn a_leading_zero_octet_cannot_inflate_a_weak_modulus() {
    // A non-conforming issuer padding a 1024-bit n to 256 octets must not pass:
    // the size is measured from the significant length, not the encoding length.
    let mut padded = vec![0u8; 128];
    padded.extend(octets(N_HEX, 256)[128..].iter().copied());
    assert_eq!(padded.len(), 256);
    assert_eq!(
        RsaPublicKey::from_be_bytes(&padded, &[0x01, 0x00, 0x01]).unwrap_err(),
        RsaError::ModulusTooSmall { bytes: 128 }
    );
}

#[test]
fn rejection_messages_name_what_went_wrong() {
    // AGENTS.md requires every error path to name the offending thing.
    assert!(format!("{}", RsaError::ModulusTooSmall { bytes: 128 }).contains("128"));
    assert!(format!("{}", RsaError::SignatureLength { got: 255, expected: 256 }).contains("255"));
    assert!(format!("{}", RsaError::PaddingRunTooShort { len: 3 }).contains('3'));
    assert!(format!("{}", RsaError::ExponentTooSmall).contains("exponent"));
    assert!(format!("{}", RsaError::SignatureOutOfRange).contains("modulus"));
}
