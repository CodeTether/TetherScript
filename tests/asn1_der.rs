//! Integration tests for the in-tree DER/ASN.1 decoder.
//!
//! Every byte array here was encoded by hand from the tag/length/value rules and
//! is annotated with its derivation, so the tests document the format as well as
//! exercise the parser.

use tetherscript::asn1::{
    der::{Reader, MAX_DEPTH},
    error::Error,
    oid, pem, tag,
};

// ---------------------------------------------------------------------------
// Primitive types, each hand-encoded.
// ---------------------------------------------------------------------------

#[test]
fn boolean_true_and_false_are_the_only_legal_encodings() {
    // 01 01 FF  =  BOOLEAN true;  01 01 00  =  BOOLEAN false
    assert!(Reader::new(&[0x01, 0x01, 0xff]).read_bool().unwrap());
    assert!(!Reader::new(&[0x01, 0x01, 0x00]).read_bool().unwrap());

    // 0x01 is a legal BER "true" but DER pins true to 0xFF.
    let ber = Reader::new(&[0x01, 0x01, 0x01]).read_bool().unwrap_err();
    assert!(matches!(ber, Error::MalformedValue { tag: 0x01, .. }));

    // Two content octets is never legal.
    let wide = Reader::new(&[0x01, 0x02, 0x00, 0xff])
        .read_bool()
        .unwrap_err();
    assert!(matches!(wide, Error::MalformedValue { .. }));
}

#[test]
fn null_must_have_zero_content_octets() {
    // 05 00  =  NULL
    assert!(Reader::new(&[0x05, 0x00]).read_null().is_ok());

    // 05 01 00  =  NULL carrying a stray content octet
    let err = Reader::new(&[0x05, 0x01, 0x00]).read_null().unwrap_err();
    assert!(matches!(err, Error::MalformedValue { tag: 0x05, .. }));
}

#[test]
fn octet_string_returns_its_content_verbatim() {
    // 04 04 DE AD BE EF  =  OCTET STRING of four bytes
    let der = [0x04, 0x04, 0xde, 0xad, 0xbe, 0xef];
    let bytes = Reader::new(&der).read_octet_string().unwrap();
    assert_eq!(bytes, &[0xde, 0xad, 0xbe, 0xef]);

    // 04 00  =  the empty OCTET STRING
    assert!(Reader::new(&[0x04, 0x00])
        .read_octet_string()
        .unwrap()
        .is_empty());
}

#[test]
fn integer_is_returned_as_raw_big_endian_bytes() {
    // 02 01 05  =  INTEGER 5
    let five = [0x02, 0x01, 0x05];
    assert_eq!(Reader::new(&five).read_integer_bytes().unwrap(), &[0x05]);

    // 02 01 00  =  INTEGER 0; a single zero octet is the minimal encoding.
    let zero = [0x02, 0x01, 0x00];
    assert_eq!(Reader::new(&zero).read_integer_bytes().unwrap(), &[0x00]);

    // 02 01 FF  =  INTEGER -1 in two's complement.
    let minus_one = [0x02, 0x01, 0xff];
    assert_eq!(Reader::new(&minus_one).read_integer_bytes().unwrap(), &[0xff]);

    // 02 02 00 80  =  INTEGER 128; the leading zero is required so that the
    // high bit of 0x80 is not read as a sign bit.
    let plus_128 = [0x02, 0x02, 0x00, 0x80];
    assert_eq!(
        Reader::new(&plus_128).read_integer_bytes().unwrap(),
        &[0x00, 0x80]
    );

    // 02 02 FF 7F  =  INTEGER -129; the leading FF is required because 0x7F
    // alone would be positive.
    let minus_129 = [0x02, 0x02, 0xff, 0x7f];
    assert_eq!(
        Reader::new(&minus_129).read_integer_bytes().unwrap(),
        &[0xff, 0x7f]
    );
}

#[test]
fn integer_rejects_illegal_leading_padding() {
    // 02 02 00 05: the 0x00 is redundant because 0x05 < 0x80.
    let padded_zero = [0x02, 0x02, 0x00, 0x05];
    let err = Reader::new(&padded_zero)
        .read_integer_bytes()
        .unwrap_err();
    assert!(matches!(
        err,
        Error::MalformedValue {
            offset: 0,
            tag: 0x02,
            ..
        }
    ));
    assert!(err.to_string().contains("leading zero"));

    // 02 03 00 00 80: two leading zeros are never minimal.
    let double_zero = [0x02, 0x03, 0x00, 0x00, 0x80];
    assert!(Reader::new(&double_zero).read_integer_bytes().is_err());

    // 02 02 FF FF: the 0xFF is redundant because the next octet is >= 0x80.
    let padded_ff = [0x02, 0x02, 0xff, 0xff];
    assert!(Reader::new(&padded_ff).read_integer_bytes().is_err());

    // 02 00: an INTEGER with no content octets.
    let empty = Reader::new(&[0x02, 0x00]).read_integer_bytes().unwrap_err();
    assert!(empty.to_string().contains("at least one content octet"));
}

#[test]
fn integer_u64_helper_strips_the_legal_pad_and_rejects_negatives() {
    // 02 03 01 00 01  =  INTEGER 65537, the usual RSA public exponent.
    let exponent = [0x02, 0x03, 0x01, 0x00, 0x01];
    assert_eq!(Reader::new(&exponent).read_u64().unwrap(), 65537);

    // 02 02 00 80  =  INTEGER 128; the legal pad octet is dropped.
    let padded = [0x02, 0x02, 0x00, 0x80];
    assert_eq!(Reader::new(&padded).read_u64().unwrap(), 128);

    // 02 01 FF  =  INTEGER -1, which is not an unsigned value.
    let negative = [0x02, 0x01, 0xff];
    assert!(Reader::new(&negative).read_u64().is_err());

    // Nine significant octets do not fit a u64.
    let wide = [
        0x02, 0x0a, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    assert!(Reader::new(&wide).read_u64().is_err());
}

#[test]
fn bit_string_exposes_its_unused_bits_octet() {
    // 03 03 00 30 00  =  BIT STRING, 0 unused bits, wrapping an empty SEQUENCE.
    let wrapping = [0x03, 0x03, 0x00, 0x30, 0x00];
    let bits = Reader::new(&wrapping).read_bit_string().unwrap();
    assert_eq!(bits.unused_bits, 0);
    assert_eq!(bits.bytes, &[0x30, 0x00]);

    // 03 02 04 F0  =  BIT STRING "1111": 4 unused bits, all of them zero.
    let padded = [0x03, 0x02, 0x04, 0xf0];
    let bits = Reader::new(&padded).read_bit_string().unwrap();
    assert_eq!(bits.unused_bits, 4);
    assert_eq!(bits.bytes, &[0xf0]);

    // 03 01 00  =  the empty BIT STRING.
    let empty = Reader::new(&[0x03, 0x01, 0x00]).read_bit_string().unwrap();
    assert_eq!(empty.unused_bits, 0);
    assert!(empty.bytes.is_empty());
}

#[test]
fn bit_string_rejects_malformed_padding() {
    // 03 00: no unused-bits octet at all.
    assert!(Reader::new(&[0x03, 0x00]).read_bit_string().is_err());

    // 03 02 08 00: a count of 8 unused bits is out of range.
    let over = [0x03, 0x02, 0x08, 0x00];
    assert!(Reader::new(&over).read_bit_string().is_err());

    // 03 01 03: a non-zero count with no value octets.
    let orphan = [0x03, 0x01, 0x03];
    assert!(Reader::new(&orphan).read_bit_string().is_err());

    // 03 02 04 F1: the low nibble must be zero when 4 bits are unused.
    let dirty = [0x03, 0x02, 0x04, 0xf1];
    let err = Reader::new(&dirty).read_bit_string().unwrap_err();
    assert!(err.to_string().contains("unused bits must be zero"));
}

#[test]
fn object_identifier_decodes_to_dotted_decimal() {
    // 06 09 2A 86 48 86 F7 0D 01 01 01  =  1.2.840.113549.1.1.1
    //   0x2A = 42 = 40*1 + 2                       -> "1.2"
    //   0x86 0x48 = (0x06 << 7) | 0x48 = 840       -> "840"
    //   0x86 0xF7 0x0D = 6*16384 + 119*128 + 13    -> 113549
    let rsa = [
        0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01,
    ];
    assert_eq!(Reader::new(&rsa).read_oid().unwrap(), oid::RSA_ENCRYPTION);

    // 06 01 28  =  1.0   (0x28 = 40 = 40*1 + 0)
    assert_eq!(Reader::new(&[0x06, 0x01, 0x28]).read_oid().unwrap(), "1.0");
    // 06 01 00  =  0.0
    assert_eq!(Reader::new(&[0x06, 0x01, 0x00]).read_oid().unwrap(), "0.0");
    // 06 01 51  =  2.1   (0x51 = 81 = 80 + 1)
    assert_eq!(Reader::new(&[0x06, 0x01, 0x51]).read_oid().unwrap(), "2.1");

    // 06 03 55 1D 0F  =  2.5.29.15, the X.509 keyUsage extension.
    let key_usage = [0x06, 0x03, 0x55, 0x1d, 0x0f];
    assert_eq!(Reader::new(&key_usage).read_oid().unwrap(), "2.5.29.15");
}

#[test]
fn object_identifier_rejects_malformed_content() {
    // 06 00: no subidentifiers.
    assert!(Reader::new(&[0x06, 0x00]).read_oid().is_err());

    // 06 02 2A 86: the final subidentifier never terminates.
    let truncated = Reader::new(&[0x06, 0x02, 0x2a, 0x86])
        .read_oid()
        .unwrap_err();
    assert!(truncated.to_string().contains("continuation bit"));

    // 06 03 2A 80 01: a 0x80 lead octet is a redundant zero group.
    let padded = Reader::new(&[0x06, 0x03, 0x2a, 0x80, 0x01])
        .read_oid()
        .unwrap_err();
    assert!(padded.to_string().contains("non-minimal"));
}

// ---------------------------------------------------------------------------
// SEQUENCE traversal.
// ---------------------------------------------------------------------------

#[test]
fn sequence_yields_its_elements_with_absolute_offsets() {
    // 30 06 02 01 05 01 01 FF  =  SEQUENCE { INTEGER 5, BOOLEAN true }
    let der = [0x30, 0x06, 0x02, 0x01, 0x05, 0x01, 0x01, 0xff];
    let mut top = Reader::new(&der);
    let mut seq = top.read_sequence().unwrap();
    assert_eq!(seq.offset(), 2);
    assert_eq!(seq.remaining(), 6);
    assert_eq!(seq.read_integer_bytes().unwrap(), &[0x05]);
    assert_eq!(seq.offset(), 5);
    assert!(seq.read_bool().unwrap());
    seq.finish().unwrap();
    top.finish().unwrap();
}

#[test]
fn nested_sequences_track_depth_and_offsets() {
    // 30 08 30 06 30 04 30 02 05 00
    //   four SEQUENCEs wrapping a NULL; each header is two bytes.
    let der = [0x30, 0x08, 0x30, 0x06, 0x30, 0x04, 0x30, 0x02, 0x05, 0x00];
    let mut reader = Reader::new(&der);
    for expected_depth in 1..=4 {
        reader = reader.read_sequence().unwrap();
        assert_eq!(reader.depth(), expected_depth);
    }
    assert_eq!(reader.offset(), 8);
    reader.read_null().unwrap();
    reader.finish().unwrap();
}

#[test]
fn tag_mismatch_names_both_tags_and_does_not_advance() {
    // 02 01 05 is an INTEGER, not a SEQUENCE.
    let der = [0x02, 0x01, 0x05];
    let mut reader = Reader::new(&der);
    let err = reader.read_sequence().unwrap_err();
    assert_eq!(
        err,
        Error::UnexpectedTag {
            offset: 0,
            expected: tag::SEQUENCE,
            found: tag::INTEGER,
        }
    );
    // The cursor is unchanged, so the caller can try a different tag.
    assert_eq!(reader.offset(), 0);
    assert_eq!(reader.read_integer_bytes().unwrap(), &[0x05]);
}

#[test]
fn descending_into_a_primitive_is_rejected() {
    let der = [0x04, 0x01, 0x00];
    let mut reader = Reader::new(&der);
    let tlv = reader.read_tlv().unwrap();
    assert!(!tlv.is_constructed());
    assert_eq!(tlv.end_offset(), 3);
    let err = reader.descend(tlv).unwrap_err();
    assert!(err.to_string().contains("primitive"));
}

#[test]
fn trailing_data_is_reported_at_the_first_leftover_byte() {
    // 30 02 05 00 followed by a stray 0x00.
    let der = [0x30, 0x02, 0x05, 0x00, 0x00];
    let mut reader = Reader::new(&der);
    reader.read_sequence().unwrap();
    assert_eq!(
        reader.finish().unwrap_err(),
        Error::TrailingData { offset: 4 }
    );
}

#[test]
fn nesting_past_the_depth_limit_is_rejected_without_a_stack_overflow() {
    // Build MAX_DEPTH + 1 nested SEQUENCEs around a NULL. Every length stays
    // below 128, so each header is exactly two bytes.
    let mut der: Vec<u8> = vec![0x05, 0x00];
    for _ in 0..=MAX_DEPTH {
        let len = u8::try_from(der.len()).expect("test document stays under 128 bytes");
        let mut wrapped = vec![0x30, len];
        wrapped.extend_from_slice(&der);
        der = wrapped;
    }

    let mut reader = Reader::new(&der);
    for _ in 0..MAX_DEPTH {
        reader = reader.read_sequence().unwrap();
    }
    assert_eq!(reader.depth(), MAX_DEPTH);
    let err = reader.read_sequence().unwrap_err();
    assert!(matches!(
        err,
        Error::DepthExceeded {
            max_depth: MAX_DEPTH,
            ..
        }
    ));
    assert!(err.to_string().contains("nesting deeper than 32"));
}

// ---------------------------------------------------------------------------
// Length-encoding rules.
// ---------------------------------------------------------------------------

#[test]
fn multi_byte_definite_lengths_are_accepted() {
    // 04 81 80 <128 bytes>  =  OCTET STRING of 128 bytes, minimal long form.
    let mut der = vec![0x04, 0x81, 0x80];
    der.extend(std::iter::repeat_n(0xaa, 128));
    assert_eq!(Reader::new(&der).read_octet_string().unwrap().len(), 128);

    // 04 82 01 00 <256 bytes>  =  OCTET STRING of 256 bytes.
    let mut der = vec![0x04, 0x82, 0x01, 0x00];
    der.extend(std::iter::repeat_n(0xbb, 256));
    assert_eq!(Reader::new(&der).read_octet_string().unwrap().len(), 256);
}

#[test]
fn indefinite_length_is_rejected() {
    // 30 80 05 00 00 00 is valid BER but forbidden in DER.
    let der = [0x30, 0x80, 0x05, 0x00, 0x00, 0x00];
    let err = Reader::new(&der).read_sequence().unwrap_err();
    assert_eq!(err, Error::IndefiniteLength { offset: 1 });
    assert!(err.to_string().contains("forbidden in DER"));
}

#[test]
fn non_minimal_lengths_are_rejected() {
    // 04 81 05 ...: 5 fits the short form, so the long form is non-minimal.
    let der = [0x04, 0x81, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
    assert_eq!(
        Reader::new(&der).read_octet_string().unwrap_err(),
        Error::NonMinimalLength { offset: 1 }
    );

    // 04 82 00 80 ...: a leading zero length octet is never minimal.
    let mut padded = vec![0x04, 0x82, 0x00, 0x80];
    padded.extend(std::iter::repeat_n(0xcc, 128));
    assert_eq!(
        Reader::new(&padded).read_octet_string().unwrap_err(),
        Error::NonMinimalLength { offset: 1 }
    );
}

#[test]
fn reserved_length_octet_is_rejected() {
    // 0xFF is reserved by X.690 and must never appear as a length octet.
    let err = Reader::new(&[0x04, 0xff, 0x00])
        .read_octet_string()
        .unwrap_err();
    assert_eq!(err, Error::ReservedLength { offset: 1 });
}

#[test]
fn a_length_claiming_more_bytes_than_exist_is_an_error_not_a_panic() {
    // 04 82 10 00 with only three content octets: claims 4096, has 3.
    let der = [0x04, 0x82, 0x10, 0x00, 0x01, 0x02, 0x03];
    let err = Reader::new(&der).read_octet_string().unwrap_err();
    assert_eq!(
        err,
        Error::LengthExceedsInput {
            offset: 4,
            length: 4096,
            available: 3,
        }
    );

    // 04 84 FF FF FF FF: a header claiming 4 GiB allocates nothing.
    let huge = [0x04, 0x84, 0xff, 0xff, 0xff, 0xff];
    let err = Reader::new(&huge).read_octet_string().unwrap_err();
    assert!(matches!(
        err,
        Error::LengthExceedsInput {
            length: 4_294_967_295,
            available: 0,
            ..
        }
    ));
}

#[test]
fn high_tag_number_form_is_rejected() {
    // 0x1F selects the multi-byte identifier form, which is out of scope.
    let err = Reader::new(&[0x1f, 0x81, 0x00, 0x00])
        .read_tlv()
        .unwrap_err();
    assert_eq!(err, Error::HighTagNumber { offset: 0 });
}

#[test]
fn truncation_at_every_boundary_is_an_error_not_a_panic() {
    // The full document: SEQUENCE { INTEGER 5, BOOLEAN true }.
    let full = [0x30, 0x06, 0x02, 0x01, 0x05, 0x01, 0x01, 0xff];
    for cut in 0..full.len() {
        let partial = &full[..cut];
        let mut reader = Reader::new(partial);
        let outcome = reader.read_sequence().and_then(|mut seq| {
            seq.read_integer_bytes()?;
            seq.read_bool()?;
            seq.finish()
        });
        assert!(outcome.is_err(), "prefix of {cut} byte(s) must not parse");
        assert!(outcome.unwrap_err().offset() <= full.len());
    }

    // Only the whole document parses.
    let mut reader = Reader::new(&full);
    let mut seq = reader.read_sequence().unwrap();
    seq.read_integer_bytes().unwrap();
    seq.read_bool().unwrap();
    seq.finish().unwrap();
    reader.finish().unwrap();
}

#[test]
fn empty_input_reports_offset_zero() {
    let empty: [u8; 0] = [];
    assert_eq!(
        Reader::new(&empty).read_tlv().unwrap_err(),
        Error::UnexpectedEnd { offset: 0 }
    );
    assert!(Reader::new(&empty).finish().is_ok());
    assert!(Reader::new(&empty).is_empty());
    assert_eq!(Reader::new(&empty).remaining(), 0);
    assert_eq!(Reader::new(&empty).depth(), 0);
}

// ---------------------------------------------------------------------------
// A real RSA SubjectPublicKeyInfo, hand-encoded with a small modulus.
// ---------------------------------------------------------------------------

/// SubjectPublicKeyInfo for a 24-bit RSA modulus 0xC0FFEE with exponent 65537.
///
/// ```text
/// 30 1F                                SubjectPublicKeyInfo, 31 content bytes
///    30 0D                             AlgorithmIdentifier, 13 content bytes
///       06 09 2A 86 48 86 F7 0D 01 01 01   OID 1.2.840.113549.1.1.1
///       05 00                          NULL parameters
///    03 0E                             BIT STRING, 14 content bytes
///       00                             0 unused bits
///       30 0B                          RSAPublicKey, 11 content bytes
///          02 04 00 C0 FF EE           INTEGER modulus (leading 0 required)
///          02 03 01 00 01              INTEGER publicExponent 65537
/// ```
///
/// A `static` rather than a `const` so that `&RSA_SPKI[..n]` borrows one fixed
/// allocation instead of a statement-scoped temporary.
static RSA_SPKI: [u8; 33] = [
    0x30, 0x1f, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05,
    0x00, 0x03, 0x0e, 0x00, 0x30, 0x0b, 0x02, 0x04, 0x00, 0xc0, 0xff, 0xee, 0x02, 0x03, 0x01, 0x00,
    0x01,
];

#[test]
fn rsa_subject_public_key_info_parses_end_to_end() {
    let mut top = Reader::new(&RSA_SPKI);
    let mut spki = top.read_sequence().unwrap();
    top.finish().unwrap();

    let mut algorithm = spki.read_sequence().unwrap();
    assert_eq!(algorithm.read_oid().unwrap(), oid::RSA_ENCRYPTION);
    algorithm.read_null().unwrap();
    algorithm.finish().unwrap();

    let key_bits = spki.read_bit_string().unwrap();
    assert_eq!(key_bits.unused_bits, 0);
    spki.finish().unwrap();

    let mut key_reader = Reader::new(key_bits.bytes);
    let mut rsa_key = key_reader.read_sequence().unwrap();
    key_reader.finish().unwrap();

    // The modulus keeps its leading zero because 0xC0 has its high bit set.
    assert_eq!(
        rsa_key.read_integer_bytes().unwrap(),
        &[0x00, 0xc0, 0xff, 0xee]
    );
    assert_eq!(rsa_key.read_u64().unwrap(), 65537);
    rsa_key.finish().unwrap();
}

#[test]
fn rsa_subject_public_key_info_truncated_at_every_boundary_never_panics() {
    for cut in 0..RSA_SPKI.len() {
        let mut top = Reader::new(&RSA_SPKI[..cut]);
        let outcome = top.read_sequence().and_then(|mut spki| {
            let mut algorithm = spki.read_sequence()?;
            algorithm.read_oid()?;
            algorithm.read_null()?;
            algorithm.finish()?;
            let bits = spki.read_bit_string()?;
            spki.finish()?;
            let mut inner = Reader::new(bits.bytes);
            let mut rsa = inner.read_sequence()?;
            rsa.read_integer_bytes()?;
            rsa.read_u64()?;
            rsa.finish()
        });
        assert!(outcome.is_err(), "prefix of {cut} byte(s) must not parse");
    }
}

// ---------------------------------------------------------------------------
// PEM armour.
// ---------------------------------------------------------------------------

/// `30 03 02 01 05` (SEQUENCE { INTEGER 5 }) base64-encodes to `MAMCAQU=`.
static SMALL_DER: [u8; 5] = [0x30, 0x03, 0x02, 0x01, 0x05];

#[test]
fn pem_round_trips_with_and_without_a_trailing_newline() {
    let with_newline = "-----BEGIN PUBLIC KEY-----\nMAMCAQU=\n-----END PUBLIC KEY-----\n";
    let block = pem::decode(with_newline).unwrap();
    assert_eq!(block.label, "PUBLIC KEY");
    assert_eq!(block.der, SMALL_DER.to_vec());

    let without_newline = with_newline.trim_end();
    assert_eq!(pem::decode(without_newline).unwrap(), block);

    // CRLF line endings decode identically; the whitespace is filtered out.
    let crlf = "-----BEGIN PUBLIC KEY-----\r\nMAMCAQU=\r\n-----END PUBLIC KEY-----\r\n";
    assert_eq!(pem::decode(crlf).unwrap().der, SMALL_DER.to_vec());

    // And the decoded DER really is SEQUENCE { INTEGER 5 }.
    let mut top = Reader::new(&block.der);
    let mut seq = top.read_sequence().unwrap();
    assert_eq!(seq.read_integer_bytes().unwrap(), &[0x05]);
    seq.finish().unwrap();
    top.finish().unwrap();
}

#[test]
fn pem_body_may_be_wrapped_across_lines() {
    // The 44-character body of the hand-encoded RSA SPKI, wrapped at 20 chars.
    let armoured = concat!(
        "-----BEGIN PUBLIC KEY-----\n",
        "MB8wDQYJKoZIhvcNAQEB\n",
        "BQADDgAwCwIEAMD/7gID\n",
        "AQAB\n",
        "-----END PUBLIC KEY-----\n",
    );
    let block = pem::decode(armoured).unwrap();
    assert_eq!(block.der, RSA_SPKI.to_vec());

    // And the decoded bytes still parse as a SubjectPublicKeyInfo.
    let mut top = Reader::new(&block.der);
    let mut spki = top.read_sequence().unwrap();
    let mut algorithm = spki.read_sequence().unwrap();
    assert_eq!(algorithm.read_oid().unwrap(), oid::RSA_ENCRYPTION);
}

#[test]
fn pem_decodes_a_certificate_label() {
    let armoured = "-----BEGIN CERTIFICATE-----\nMAMCAQU=\n-----END CERTIFICATE-----";
    assert_eq!(pem::decode(armoured).unwrap().label, "CERTIFICATE");
}

#[test]
fn pem_rejects_missing_or_mismatched_armour() {
    // No BEGIN line at all.
    let err = pem::decode("MAMCAQU=").unwrap_err();
    assert!(matches!(err, Error::Pem { offset: 0, .. }));
    assert!(err.to_string().contains("BEGIN"));

    // BEGIN with no END.
    assert!(pem::decode("-----BEGIN PUBLIC KEY-----\nMAMCAQU=\n").is_err());

    // Labels disagree, so the END marker for "PUBLIC KEY" is never found.
    let mismatched = "-----BEGIN PUBLIC KEY-----\nMAMCAQU=\n-----END PRIVATE KEY-----\n";
    assert!(pem::decode(mismatched).is_err());

    // BEGIN line missing its closing dashes.
    assert!(pem::decode("-----BEGIN PUBLIC KEY\nMAMCAQU=\n").is_err());

    // Completely empty input.
    assert!(pem::decode("").is_err());
}

#[test]
fn pem_rejects_an_invalid_base64_body() {
    // '!' is not in the base64 alphabet.
    let bad_char = "-----BEGIN PUBLIC KEY-----\nMAMC!QU=\n-----END PUBLIC KEY-----\n";
    let err = pem::decode(bad_char).unwrap_err();
    assert!(matches!(err, Error::Pem { .. }));

    // Five characters is not a whole base64 quantum.
    let bad_len = "-----BEGIN PUBLIC KEY-----\nMAMCA\n-----END PUBLIC KEY-----\n";
    assert!(pem::decode(bad_len).is_err());
}

#[test]
fn every_error_reports_a_named_cause_and_a_usable_offset() {
    let cases: Vec<Error> = vec![
        Reader::new(&[]).read_tlv().unwrap_err(),
        Reader::new(&[0x04, 0x05, 0x00])
            .read_octet_string()
            .unwrap_err(),
        Reader::new(&[0x30, 0x80]).read_sequence().unwrap_err(),
        Reader::new(&[0x04, 0x81, 0x01, 0x00])
            .read_octet_string()
            .unwrap_err(),
        Reader::new(&[0x04, 0xff]).read_octet_string().unwrap_err(),
        Reader::new(&[0x1f, 0x00]).read_tlv().unwrap_err(),
        Reader::new(&[0x02, 0x01, 0x00]).read_sequence().unwrap_err(),
        Reader::new(&[0x02, 0x02, 0x00, 0x01])
            .read_integer_bytes()
            .unwrap_err(),
        pem::decode("nothing here").unwrap_err(),
    ];
    for err in cases {
        let text = err.to_string();
        assert!(text.starts_with("asn1: "), "unhelpful message: {text}");
        assert!(text.contains("offset"), "message omits an offset: {text}");
        assert!(err.offset() < 64);
    }
}
