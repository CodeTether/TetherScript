//! Unit coverage for the DER `DigestInfo` prefix table.
//!
//! The prefixes are frozen constants quoted from RFC 8017 section 9.2, note 1,
//! so the useful assertions are *internal consistency* ones: each prefix must be
//! a well-formed DER `SEQUENCE` whose declared lengths agree with the digest
//! length the enum reports. A typo in a table entry would otherwise only surface
//! as a mysterious verification failure against a real signer.
//!
//! The integrator wires this with `#[cfg(test)] mod digestinfo_tests;`.

use super::digestinfo::DigestAlgorithm;

/// Every algorithm, so no test can silently skip one.
const ALL: [DigestAlgorithm; 4] = [
    DigestAlgorithm::Sha1,
    DigestAlgorithm::Sha256,
    DigestAlgorithm::Sha384,
    DigestAlgorithm::Sha512,
];

#[test]
fn every_prefix_is_a_der_sequence_with_a_consistent_outer_length() {
    for alg in ALL {
        let prefix = alg.der_prefix();
        // 0x30 is the DER tag for SEQUENCE, and the short-form length that
        // follows counts every octet after it: the inner AlgorithmIdentifier,
        // the OCTET STRING header, and the digest.
        assert_eq!(prefix[0], 0x30, "{alg:?} must start a SEQUENCE");
        let declared = prefix[1] as usize;
        assert_eq!(declared, alg.encoded_len() - 2, "{alg:?} outer length");
    }
}

#[test]
fn every_prefix_ends_with_an_octet_string_header_of_the_digest_length() {
    for alg in ALL {
        let prefix = alg.der_prefix();
        // 0x04 is the DER tag for OCTET STRING; its length must be the digest
        // length, or the digest and its declared size disagree.
        assert_eq!(prefix[prefix.len() - 2], 0x04, "{alg:?} OCTET STRING tag");
        assert_eq!(*prefix.last().unwrap() as usize, alg.digest_len());
    }
}

#[test]
fn digest_lengths_are_the_published_output_sizes() {
    assert_eq!(DigestAlgorithm::Sha1.digest_len(), 20);
    assert_eq!(DigestAlgorithm::Sha256.digest_len(), 32);
    assert_eq!(DigestAlgorithm::Sha384.digest_len(), 48);
    assert_eq!(DigestAlgorithm::Sha512.digest_len(), 64);
}

#[test]
fn the_sha2_family_prefixes_share_one_oid_arc_and_differ_only_in_the_last_arc() {
    // 2.16.840.1.101.3.4.2.{1,2,3} encodes as 608648016503040 2 0{1,2,3}, so the
    // prefixes must agree everywhere except the final OID arc and the two length
    // octets. A copy-paste error in the table would break this.
    let s256 = DigestAlgorithm::Sha256.der_prefix();
    let s384 = DigestAlgorithm::Sha384.der_prefix();
    let s512 = DigestAlgorithm::Sha512.der_prefix();
    assert_eq!(s256[2..14], s384[2..14]);
    assert_eq!(s256[2..14], s512[2..14]);
    assert_eq!((s256[14], s384[14], s512[14]), (0x01, 0x02, 0x03));
}

#[test]
fn no_two_algorithms_share_a_prefix() {
    // Distinctness is what makes the DigestInfo check an algorithm check.
    for (index, first) in ALL.iter().enumerate() {
        for second in &ALL[index + 1..] {
            assert_ne!(first.der_prefix(), second.der_prefix());
        }
    }
}
