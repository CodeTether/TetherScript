//! Pure unit coverage for binary-format PostgreSQL decoding and encoding.
//!
//! Every test here operates on **byte slices with embedded fixtures**. There is no
//! live server, no socket, and no environment gate, so these run in the default
//! hermetic `cargo test`. That is deliberate: wire-format decoding is exactly the kind
//! of pure byte-in/value-out logic a fixture proves better than a live server does,
//! because a fixture can express a *malformed* frame — a truncated field, a bad sign
//! word, a 2-D array — that a real server will never send.
//!
//! `tests/postgres_live.rs` remains the proof of protocol *negotiation*; this file is
//! the proof of protocol *parsing*.
//!
//! Fixtures were generated and cross-checked with the scripts in `artifacts/`:
//! `pgbinary_verify_civil.py` (292 194 days of calendar round trips against Python's
//! `datetime`), `pgbinary_verify_numeric.py` (exact `numeric` rendering against
//! Python's `decimal`), and `pgbinary_verify_roundtrip.py` (encode/decode fidelity).

use std::cell::RefCell;
use std::rc::Rc;

use tetherscript::postgres::binary::{
    civil_from_days, date_unix_days, days_from_civil, decode_field, decode_nullable, encode_param,
    format_codes, numeric_to_string, oid, supports, timestamp_unix_micros, DecodeError,
    FORMAT_BINARY, FORMAT_TEXT, PG_EPOCH_UNIX_DAYS, PG_EPOCH_UNIX_MICROS, PG_EPOCH_UNIX_SECONDS,
};
use tetherscript::value::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Decode and expect success, naming the type on failure.
fn decode(type_oid: u32, body: &[u8]) -> Value {
    decode_field(type_oid, body)
        .unwrap_or_else(|error| panic!("OID {type_oid} should decode: {error}"))
}

/// The decoded value as a string, for the types that render as text.
fn text(type_oid: u32, body: &[u8]) -> String {
    match decode(type_oid, body) {
        Value::Str(value) => value.as_ref().clone(),
        other => panic!("expected a str, got {}", other.type_name()),
    }
}

/// A tetherscript string value.
fn str_value(text: &str) -> Value {
    Value::Str(Rc::new(text.to_string()))
}

/// A tetherscript list value.
fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(items)))
}

/// The elements of a decoded list.
fn items(value: &Value) -> Vec<Value> {
    match value {
        Value::List(items) => items.borrow().clone(),
        other => panic!("expected a list, got {}", other.type_name()),
    }
}

/// Encode and expect a present (non-NULL) value.
fn encode(type_oid: u32, value: &Value) -> Vec<u8> {
    encode_param(type_oid, value)
        .unwrap_or_else(|error| panic!("OID {type_oid} should encode: {error}"))
        .expect("a non-nil value must encode to Some")
}

// ===========================================================================
// Endianness: every multi-byte value is big-endian (network byte order).
// ===========================================================================

/// The canonical proof: 0x00010203 is 66051 big-endian, 50462976 little-endian.
/// If this ever reads 50462976, a `from_le_bytes` has crept in.
#[test]
fn int4_is_big_endian_not_little_endian() {
    assert_eq!(decode(oid::INT4, &[0, 1, 2, 3]), Value::Int(66_051));
    assert_ne!(
        decode(oid::INT4, &[0, 1, 2, 3]),
        Value::Int(50_462_976),
        "a little-endian read would produce this byte-swapped value"
    );
}

#[test]
fn integers_decode_at_every_width_including_the_extremes() {
    assert_eq!(decode(oid::INT2, &[255, 253]), Value::Int(-3));
    assert_eq!(decode(oid::INT2, &[0x7F, 0xFF]), Value::Int(32_767));
    assert_eq!(decode(oid::INT2, &[0x80, 0x00]), Value::Int(-32_768));
    assert_eq!(decode(oid::INT4, &[0xFF; 4]), Value::Int(-1));
    assert_eq!(
        decode(oid::INT8, &[0x7F, 255, 255, 255, 255, 255, 255, 255]),
        Value::Int(i64::MAX)
    );
    assert_eq!(
        decode(oid::INT8, &[0x80, 0, 0, 0, 0, 0, 0, 0]),
        Value::Int(i64::MIN)
    );
}

/// `oid` shares int4's width but is unsigned, so it must never decode negative.
#[test]
fn oid_is_unsigned_despite_sharing_int4s_width() {
    assert_eq!(decode(oid::OID, &[0xFF; 4]), Value::Int(4_294_967_295));
    assert_eq!(decode(oid::INT4, &[0xFF; 4]), Value::Int(-1));
}

#[test]
fn floats_are_big_endian_ieee754_bit_patterns() {
    // 1.5f32 == 0x3FC00000, -2.25f64 == 0xC002000000000000
    assert_eq!(decode(oid::FLOAT4, &[0x3F, 0xC0, 0, 0]), Value::Float(1.5));
    assert_eq!(
        decode(oid::FLOAT8, &[0xC0, 0x02, 0, 0, 0, 0, 0, 0]),
        Value::Float(-2.25)
    );
    // float4 widening to f64 is exact for every f32.
    assert_eq!(decode(oid::FLOAT4, &[0, 0, 0, 0]), Value::Float(0.0));
}

#[test]
fn float_nan_and_infinity_survive_as_bit_patterns() {
    let nan = decode(oid::FLOAT8, &[0x7F, 0xF8, 0, 0, 0, 0, 0, 0]);
    match nan {
        Value::Float(value) => assert!(value.is_nan(), "expected NaN, got {value}"),
        other => panic!("expected a float, got {}", other.type_name()),
    }
    let inf = decode(oid::FLOAT8, &[0x7F, 0xF0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(inf, Value::Float(f64::INFINITY));
}

#[test]
fn bool_accepts_only_zero_and_one() {
    assert_eq!(decode(oid::BOOL, &[0]), Value::Bool(false));
    assert_eq!(decode(oid::BOOL, &[1]), Value::Bool(true));
    let error = decode_field(oid::BOOL, &[2]).expect_err("byte 2 is not a bool");
    assert!(error.to_string().contains("bool"), "got: {error}");
}

// ===========================================================================
// Text-like types, including the jsonb version byte.
// ===========================================================================

#[test]
fn text_family_oids_all_decode_as_utf8_strings() {
    for type_oid in [oid::TEXT, oid::VARCHAR, oid::BPCHAR, oid::NAME, oid::XML] {
        assert_eq!(text(type_oid, "héllo".as_bytes()), "héllo");
    }
}

#[test]
fn invalid_utf8_is_a_named_error_not_a_replacement_character() {
    // 0xFF is never valid UTF-8. Lossy conversion would yield U+FFFD and hide it.
    let error = decode_field(oid::TEXT, &[0xFF]).expect_err("0xFF is not UTF-8");
    assert!(error.to_string().contains("UTF-8"), "got: {error}");
    assert!(!error.needs_text_fallback());
}

/// jsonb carries a leading version byte that json does not. Both are handled.
#[test]
fn jsonb_strips_its_version_byte_and_json_does_not_have_one() {
    let document = br#"{"a":1}"#;
    assert_eq!(text(oid::JSON, document), r#"{"a":1}"#);

    let mut jsonb = vec![1u8]; // the jsonb binary version byte
    jsonb.extend_from_slice(document);
    assert_eq!(text(oid::JSONB, &jsonb), r#"{"a":1}"#);

    // Feeding a jsonb body to the json decoder would leave the control byte in
    // place, which is exactly the confusing failure the split prevents.
    assert_ne!(text(oid::JSON, &jsonb), r#"{"a":1}"#);
}

#[test]
fn an_unknown_jsonb_version_is_rejected_by_name() {
    let error = decode_field(oid::JSONB, &[9, b'{', b'}']).expect_err("version 9 is unknown");
    assert!(error.to_string().contains("jsonb"), "got: {error}");
    assert!(
        error.to_string().contains('9'),
        "should name it, got: {error}"
    );
}

#[test]
fn bytea_is_raw_bytes_with_no_charset_assumption() {
    match decode(oid::BYTEA, &[0, 0xFF, 0x80]) {
        Value::Bytes(bytes) => assert_eq!(*bytes.borrow(), vec![0, 0xFF, 0x80]),
        other => panic!("expected bytes, got {}", other.type_name()),
    }
    // An empty bytea is a present, zero-length value.
    match decode(oid::BYTEA, &[]) {
        Value::Bytes(bytes) => assert!(bytes.borrow().is_empty()),
        other => panic!("expected bytes, got {}", other.type_name()),
    }
}

#[test]
fn uuid_renders_16_bytes_in_canonical_hyphenated_form() {
    let bytes: Vec<u8> = (0u8..16).collect();
    assert_eq!(
        text(oid::UUID, &bytes),
        "00010203-0405-0607-0809-0a0b0c0d0e0f"
    );
}

// ===========================================================================
// Epochs: PostgreSQL counts from 2000-01-01, not 1970-01-01.
// ===========================================================================

/// The epoch shift is 946_684_800 seconds. Pinned two independent ways: by the
/// leap-day arithmetic that derives it, and against a known instant. Getting this
/// wrong produces dates 30 years off that still look perfectly well-formed.
#[test]
fn the_epoch_shift_is_exactly_thirty_years_plus_seven_leap_days() {
    // 30 years of 365 days, plus 1972, 1976, 1980, 1984, 1988, 1992, 1996.
    assert_eq!(PG_EPOCH_UNIX_DAYS, 30 * 365 + 7);
    assert_eq!(PG_EPOCH_UNIX_DAYS, 10_957);
    assert_eq!(PG_EPOCH_UNIX_SECONDS, 946_684_800);
    assert_eq!(
        PG_EPOCH_UNIX_SECONDS,
        PG_EPOCH_UNIX_DAYS as i64 * 86_400,
        "the seconds constant must be the day constant times 86400"
    );
    assert_eq!(PG_EPOCH_UNIX_MICROS, PG_EPOCH_UNIX_SECONDS * 1_000_000);
    // 2000-01-01 really is 10957 days after 1970-01-01.
    assert_eq!(civil_from_days(PG_EPOCH_UNIX_DAYS as i64), (2000, 1, 1));
}

/// The counter is MICROSECONDS since 2000-01-01, not seconds since 1970. Both
/// mistakes are checked for explicitly, because both produce valid-looking dates.
#[test]
fn timestamp_epoch_conversion_holds_in_both_directions() {
    // 2024-01-15T10:30:00Z is Unix 1_705_314_600 s == 758_629_800_000_000 pg µs.
    let pg_micros = 758_629_800_000_000i64;
    let unix_micros = 1_705_314_600_000_000i64;
    assert_eq!(timestamp_unix_micros(pg_micros), unix_micros);
    assert_eq!(
        timestamp_unix_micros(pg_micros) - PG_EPOCH_UNIX_MICROS,
        pg_micros
    );

    // The PostgreSQL zero point is 2000-01-01, not 1970-01-01.
    assert_eq!(timestamp_unix_micros(0), 946_684_800_000_000);
    assert_ne!(
        timestamp_unix_micros(0),
        0,
        "0 must not mean the Unix epoch"
    );

    // Reading the counter as seconds would land ~24 million years out.
    let as_if_seconds = pg_micros + PG_EPOCH_UNIX_SECONDS;
    assert_ne!(as_if_seconds, unix_micros / 1_000_000);
}

#[test]
fn timestamptz_decodes_a_known_instant_and_appends_z() {
    let body = 758_629_800_000_000i64.to_be_bytes();
    assert_eq!(text(oid::TIMESTAMPTZ, &body), "2024-01-15T10:30:00Z");
    // Plain timestamp has the same layout and no zone marker.
    assert_eq!(text(oid::TIMESTAMP, &body), "2024-01-15T10:30:00");
}

/// Skipping the epoch shift would render 1994-01-15 — plausible, and wrong.
#[test]
fn forgetting_the_epoch_shift_would_be_off_by_thirty_years() {
    let body = 758_629_800_000_000i64.to_be_bytes();
    let decoded = text(oid::TIMESTAMPTZ, &body);
    assert!(decoded.starts_with("2024-"), "got: {decoded}");
    assert!(!decoded.starts_with("1994-"), "epoch shift was skipped");
}

#[test]
fn timestamp_zero_is_the_year_2000_not_1970() {
    assert_eq!(
        text(oid::TIMESTAMPTZ, &0i64.to_be_bytes()),
        "2000-01-01T00:00:00Z"
    );
}

#[test]
fn timestamp_keeps_microsecond_precision() {
    let body = 758_629_800_123_456i64.to_be_bytes();
    assert_eq!(text(oid::TIMESTAMPTZ, &body), "2024-01-15T10:30:00.123456Z");
    // A trailing-zero fraction is trimmed but not lost.
    let half = 758_629_800_500_000i64.to_be_bytes();
    assert_eq!(text(oid::TIMESTAMPTZ, &half), "2024-01-15T10:30:00.5Z");
}

/// A pre-1970 instant has a negative Unix counter. Truncating division would give a
/// negative time-of-day and render nonsense; Euclidean division must be used.
#[test]
fn a_pre_1970_timestamp_renders_with_a_positive_time_of_day() {
    // 1969-07-20T20:17:40Z, the Apollo 11 landing.
    let body = (-960_867_740_000_000i64).to_be_bytes();
    assert_eq!(text(oid::TIMESTAMPTZ, &body), "1969-07-20T20:17:40Z");
}

#[test]
fn timestamp_infinities_render_by_name_not_as_an_absurd_year() {
    assert_eq!(text(oid::TIMESTAMPTZ, &i64::MAX.to_be_bytes()), "infinity");
    assert_eq!(text(oid::TIMESTAMPTZ, &i64::MIN.to_be_bytes()), "-infinity");
    assert_eq!(text(oid::DATE, &i32::MAX.to_be_bytes()), "infinity");
    assert_eq!(text(oid::DATE, &i32::MIN.to_be_bytes()), "-infinity");
}

/// date is DAYS since 2000-01-01, so the same 30-year shift applies.
#[test]
fn date_applies_the_same_epoch_shift_in_days() {
    assert_eq!(date_unix_days(0), PG_EPOCH_UNIX_DAYS);
    assert_eq!(date_unix_days(8_780), 19_737);
    assert_eq!(text(oid::DATE, &8_780i32.to_be_bytes()), "2024-01-15");
    assert_eq!(text(oid::DATE, &0i32.to_be_bytes()), "2000-01-01");
    // A negative day count reaches back before 2000.
    assert_eq!(text(oid::DATE, &(-1i32).to_be_bytes()), "1999-12-31");
}

#[test]
fn the_calendar_handles_leap_years_and_the_century_rules() {
    // 2000 is a leap year (divisible by 400); 1900 is not (divisible by 100).
    assert_eq!(civil_from_days(days_from_civil(2000, 2, 29)), (2000, 2, 29));
    assert_eq!(
        days_from_civil(1900, 3, 1) - days_from_civil(1900, 2, 28),
        1
    );
    assert_eq!(
        days_from_civil(2000, 3, 1) - days_from_civil(2000, 2, 28),
        2
    );
    // Round-trip across a spread of boundaries.
    for days in [-100_000i64, -1, 0, 1, 10_957, 19_737, 100_000] {
        let (year, month, day) = civil_from_days(days);
        assert_eq!(days_from_civil(year, month, day), days, "at day {days}");
    }
}

#[test]
fn time_is_microseconds_since_midnight_with_no_epoch() {
    assert_eq!(text(oid::TIME, &0i64.to_be_bytes()), "00:00:00");
    assert_eq!(
        text(oid::TIME, &37_800_123_456i64.to_be_bytes()),
        "10:30:00.123456"
    );
    assert_eq!(
        text(oid::TIME, &86_399_000_000i64.to_be_bytes()),
        "23:59:59"
    );
}

// ===========================================================================
// numeric: exact decimal, never through f64.
// ===========================================================================

/// The load-bearing case. 12345.6789 as base-10000 groups [1, 2345, 6789] with
/// weight 1 and dscale 4. Note the interior group 2345 must be zero-padded to four
/// digits; only the leading group is unpadded.
#[test]
fn numeric_decodes_a_multi_group_value_exactly() {
    let body = [0, 3, 0, 1, 0, 0, 0, 4, 0, 1, 9, 41, 26, 133];
    assert_eq!(numeric_to_string(&body).unwrap(), "12345.6789");
    assert_eq!(decode(oid::NUMERIC, &body), str_value("12345.6789"));
}

/// A money value must not round-trip through a double. 19.99 is unrepresentable in
/// binary floating point, so if this ever yields 19.989999999999998 an f64 crept in.
#[test]
fn numeric_does_not_round_trip_through_a_float() {
    // ndigits 2, weight 0, sign +, dscale 2, groups [19, 9900]
    let body = [0, 2, 0, 0, 0, 0, 0, 2, 0, 19, 38, 172];
    let decoded = numeric_to_string(&body).unwrap();
    assert_eq!(decoded, "19.99");
    assert!(
        !decoded.contains("9999999"),
        "float contamination: {decoded}"
    );
    // And it is exactly the string, not a float formatted back.
    assert_eq!(decoded.len(), 5);
}

#[test]
fn numeric_zero_has_no_digit_groups_at_all() {
    // ndigits 0, weight 0, sign +, dscale 0
    assert_eq!(numeric_to_string(&[0, 0, 0, 0, 0, 0, 0, 0]).unwrap(), "0");
    // Zero with a display scale keeps its scale: 0.000, not 0.
    assert_eq!(
        numeric_to_string(&[0, 0, 0, 0, 0, 0, 0, 3]).unwrap(),
        "0.000"
    );
}

#[test]
fn numeric_nan_is_the_sign_word_not_a_digit_pattern() {
    // sign 0xC000, no digits.
    assert_eq!(
        numeric_to_string(&[0, 0, 0, 0, 0xC0, 0, 0, 0]).unwrap(),
        "NaN"
    );
    assert_eq!(
        decode(oid::NUMERIC, &[0, 0, 0, 0, 0xC0, 0, 0, 0]),
        str_value("NaN")
    );
}

#[test]
fn numeric_infinities_use_their_own_sign_words() {
    assert_eq!(
        numeric_to_string(&[0, 0, 0, 0, 0xD0, 0, 0, 0]).unwrap(),
        "Infinity"
    );
    assert_eq!(
        numeric_to_string(&[0, 0, 0, 0, 0xF0, 0, 0, 0]).unwrap(),
        "-Infinity"
    );
}

/// Negative is a sign word, not a two's-complement digit; digits stay non-negative.
#[test]
fn numeric_negative_comes_from_the_sign_word() {
    // ndigits 1, weight -1, sign 0x4000, dscale 2, groups [5000] => -0.50
    let body = [0, 1, 255, 255, 0x40, 0, 0, 2, 19, 136];
    assert_eq!(numeric_to_string(&body).unwrap(), "-0.50");
    // The same digits with a positive sign word.
    let positive = [0, 1, 255, 255, 0, 0, 0, 2, 19, 136];
    assert_eq!(numeric_to_string(&positive).unwrap(), "0.50");
}

/// A trailing zero in the scale is meaningful: 0.50 is a price, 0.5 is not the same
/// rendering. dscale must be honoured rather than trimmed.
#[test]
fn numeric_preserves_its_display_scale_including_trailing_zeros() {
    let body = [0, 1, 255, 255, 0, 0, 0, 2, 19, 136];
    assert_eq!(numeric_to_string(&body).unwrap(), "0.50");
    assert_ne!(numeric_to_string(&body).unwrap(), "0.5");
}

#[test]
fn numeric_handles_a_high_scale_far_beyond_f64_precision() {
    // 1e-30: ndigits 1, weight -8, dscale 30, groups [100]. An f64 has ~15-17
    // significant digits, so a float path could not represent this at all.
    let body = [0, 1, 255, 248, 0, 0, 0, 30, 0, 100];
    assert_eq!(
        numeric_to_string(&body).unwrap(),
        "0.000000000000000000000000000001"
    );
}

/// An interior zero group must render as 0007, not collapse into the digit above it.
#[test]
fn numeric_zero_pads_interior_digit_groups() {
    // 7.0007: ndigits 2, weight 0, dscale 4, groups [7, 7]
    let body = [0, 2, 0, 0, 0, 0, 0, 4, 0, 7, 0, 7];
    assert_eq!(numeric_to_string(&body).unwrap(), "7.0007");
    assert_ne!(numeric_to_string(&body).unwrap(), "7.7");
}

#[test]
fn numeric_handles_a_large_integer_beyond_i64() {
    // 123456789012345678901234567890, weight 7, 8 groups, dscale 0.
    let body = [
        0, 8, 0, 7, 0, 0, 0, 0, 0, 12, 13, 128, 30, 210, 4, 210, 22, 46, 35, 52, 13, 128, 30, 210,
    ];
    assert_eq!(
        numeric_to_string(&body).unwrap(),
        "123456789012345678901234567890"
    );
}

#[test]
fn an_unrecognised_numeric_sign_word_is_rejected_by_name() {
    let error = numeric_to_string(&[0, 0, 0, 0, 0xAB, 0xCD, 0, 0])
        .expect_err("0xABCD is not a documented sign word");
    assert!(
        matches!(error, DecodeError::BadNumericSign { .. }),
        "got: {error}"
    );
    assert!(error.to_string().contains("sign"), "got: {error}");
}

#[test]
fn a_negative_numeric_digit_count_is_rejected_rather_than_cast() {
    // A negative ndigits cast to usize would become an enormous allocation.
    let error = numeric_to_string(&[255, 255, 0, 0, 0, 0, 0, 0]).expect_err("ndigits -1");
    assert!(error.to_string().contains("numeric"), "got: {error}");
}

#[test]
fn an_out_of_range_numeric_digit_group_is_rejected() {
    // 10000 exceeds the base-10000 maximum of 9999.
    let body = [0, 1, 0, 0, 0, 0, 0, 0, 39, 16];
    let error = numeric_to_string(&body).expect_err("10000 is not a valid group");
    assert!(error.to_string().contains("9999"), "got: {error}");
}

// ===========================================================================
// NULL (length -1) versus a zero-length value.
// ===========================================================================

/// The distinction that turns an empty string into nil if conflated. A field length
/// of -1 is SQL NULL and carries no bytes; a length of 0 is a present empty value.
#[test]
fn null_and_empty_are_different_values_for_every_text_type() {
    for type_oid in [oid::TEXT, oid::VARCHAR, oid::JSON] {
        assert_eq!(
            decode_nullable(type_oid, None).unwrap(),
            Value::Nil,
            "length -1 must be nil"
        );
        assert_eq!(
            decode_nullable(type_oid, Some(&[])).unwrap(),
            str_value(""),
            "length 0 must be the empty string"
        );
        assert_ne!(
            decode_nullable(type_oid, None).unwrap(),
            decode_nullable(type_oid, Some(&[])).unwrap()
        );
    }
}

#[test]
fn an_empty_bytea_is_not_null() {
    let empty = decode_nullable(oid::BYTEA, Some(&[])).unwrap();
    assert_ne!(empty, Value::Nil);
    match empty {
        Value::Bytes(bytes) => assert!(bytes.borrow().is_empty()),
        other => panic!("expected bytes, got {}", other.type_name()),
    }
}

/// A NULL never fails, whatever its declared type — even a fixed-width one whose
/// decoder would reject a zero-length body.
#[test]
fn a_null_succeeds_for_a_fixed_width_type_whose_body_would_be_truncated() {
    assert_eq!(decode_nullable(oid::INT4, None).unwrap(), Value::Nil);
    assert_eq!(decode_nullable(oid::TIMESTAMPTZ, None).unwrap(), Value::Nil);
    // But a genuinely zero-length int4 body is truncated, not NULL.
    assert!(decode_nullable(oid::INT4, Some(&[])).is_err());
}

// ===========================================================================
// Robustness: truncated and over-long input rejected BY NAME, never a panic.
// ===========================================================================

/// A decoder reads bytes from a network peer, so every length must be checked before
/// it is trusted. Each of these would be a slice-index panic without that check, and
/// each must name the type it failed to read rather than saying "decode error".
#[test]
fn every_truncated_field_names_the_type_it_failed_to_read() {
    let cases: &[(u32, &str, &[u8])] = &[
        (oid::BOOL, "bool", &[]),
        (oid::INT2, "int2", &[0]),
        (oid::INT4, "int4", &[0, 1]),
        (oid::INT8, "int8", &[0, 1, 2]),
        (oid::FLOAT4, "float4", &[0]),
        (oid::FLOAT8, "float8", &[0, 1, 2, 3]),
        (oid::DATE, "date", &[0, 1]),
        (oid::TIME, "time", &[0]),
        (oid::TIMESTAMP, "timestamp", &[0, 1, 2, 3, 4]),
        (oid::UUID, "uuid", &[0, 1, 2]),
        (oid::NUMERIC, "numeric", &[0, 1]),
        (oid::JSONB, "jsonb", &[]),
    ];
    for (type_oid, name, body) in cases {
        let error = decode_field(*type_oid, body)
            .expect_err(&format!("OID {type_oid} must reject a short body"));
        let message = error.to_string();
        assert!(
            message.contains(name),
            "error for OID {type_oid} should name {name:?}, got: {message}"
        );
        assert!(
            !error.needs_text_fallback(),
            "a truncated field is a protocol fault, not a fallback case"
        );
    }
}

/// An over-long body means the field was not the type its OID claimed. Ignoring the
/// tail would let a mismatched OID decode to a plausible wrong value.
#[test]
fn an_over_long_field_is_rejected_rather_than_silently_truncated() {
    let error = decode_field(oid::INT4, &[0, 0, 0, 1, 99]).expect_err("5 bytes is not an int4");
    assert!(
        matches!(error, DecodeError::Overlong { .. }),
        "got: {error}"
    );
    assert!(error.to_string().contains("int4"), "got: {error}");

    let long_uuid: Vec<u8> = (0u8..17).collect();
    assert!(decode_field(oid::UUID, &long_uuid).is_err());
    assert!(decode_field(oid::BOOL, &[1, 1]).is_err());
}

/// Exhaustive proof of the no-panic invariant: every supported OID, every body length
/// from 0 to 40 bytes. Any of these panicking would take down the host process.
#[test]
fn no_decoder_panics_on_any_prefix_of_any_length() {
    let oids = [
        oid::BOOL,
        oid::BYTEA,
        oid::CHAR,
        oid::NAME,
        oid::INT8,
        oid::INT2,
        oid::INT4,
        oid::TEXT,
        oid::OID,
        oid::JSON,
        oid::XML,
        oid::FLOAT4,
        oid::FLOAT8,
        oid::BPCHAR,
        oid::VARCHAR,
        oid::DATE,
        oid::TIME,
        oid::TIMESTAMP,
        oid::TIMESTAMPTZ,
        oid::NUMERIC,
        oid::UUID,
        oid::JSONB,
        oid::INT4_ARRAY,
        oid::TEXT_ARRAY,
        oid::NUMERIC_ARRAY,
        oid::TIMESTAMPTZ_ARRAY,
    ];
    let filler: Vec<u8> = (0u8..40).collect();
    for type_oid in oids {
        for len in 0..=filler.len() {
            // The result is irrelevant; not panicking is the assertion.
            let _ = decode_field(type_oid, &filler[..len]);
            let _ = decode_field(type_oid, &vec![0xFF; len]);
            let _ = decode_field(type_oid, &vec![0x00; len]);
        }
    }
}

// ===========================================================================
// Arrays: header, NULL element, dimension rejection.
// ===========================================================================

/// Header: ndim 1, has_null 1, elem oid 23, length 2, lower bound 1. Then a 4-byte
/// int4 and a length of -1 for the NULL element, which carries no bytes.
#[test]
fn an_array_with_a_null_element_keeps_the_null_distinct() {
    let body = [
        0, 0, 0, 1, // ndim = 1
        0, 0, 0, 1, // has_null = 1 (advisory)
        0, 0, 0, 23, // element oid = int4
        0, 0, 0, 2, // length = 2
        0, 0, 0, 1, // lower bound = 1
        0, 0, 0, 4, 0, 0, 0, 7, // element 0: 4 bytes, int4 7
        255, 255, 255, 255, // element 1: length -1, SQL NULL
    ];
    let decoded = items(&decode(oid::INT4_ARRAY, &body));
    assert_eq!(decoded, vec![Value::Int(7), Value::Nil]);
}

/// A NULL element and an empty-string element are different, one level down.
#[test]
fn an_array_distinguishes_a_null_element_from_an_empty_one() {
    let body = [
        0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 25, // text[]
        0, 0, 0, 2, 0, 0, 0, 1, //
        0, 0, 0, 0, // element 0: length 0, the empty string
        255, 255, 255, 255, // element 1: length -1, NULL
    ];
    let decoded = items(&decode(oid::TEXT_ARRAY, &body));
    assert_eq!(decoded, vec![str_value(""), Value::Nil]);
    assert_ne!(decoded[0], decoded[1]);
}

/// An empty array is ndim 0 with NO dimension block. That is a different message
/// from ndim 1 with length 0.
#[test]
fn an_empty_array_has_no_dimension_block() {
    let body = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23];
    assert_eq!(items(&decode(oid::INT4_ARRAY, &body)), Vec::<Value>::new());
}

#[test]
fn arrays_of_every_common_scalar_decode_through_their_element_decoder() {
    // One 8-byte timestamptz element: 2024-01-15T10:30:00Z.
    let mut body = vec![
        0, 0, 0, 1, 0, 0, 0, 0, //
    ];
    body.extend_from_slice(&oid::TIMESTAMPTZ.to_be_bytes());
    body.extend_from_slice(&1i32.to_be_bytes());
    body.extend_from_slice(&1i32.to_be_bytes());
    body.extend_from_slice(&8i32.to_be_bytes());
    body.extend_from_slice(&758_629_800_000_000i64.to_be_bytes());
    assert_eq!(
        items(&decode(oid::TIMESTAMPTZ_ARRAY, &body)),
        vec![str_value("2024-01-15T10:30:00Z")]
    );
}

/// A dimension count this decoder will not handle must be REJECTED, not misread.
/// Flattening {{1,2},{3,4}} into [1,2,3,4] discards shape irrecoverably.
#[test]
fn a_multi_dimensional_array_is_rejected_rather_than_flattened() {
    let body = [
        0, 0, 0, 2, // ndim = 2
        0, 0, 0, 0, 0, 0, 0, 23, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1,
    ];
    let error = decode_field(oid::INT4_ARRAY, &body).expect_err("2-D must be rejected");
    assert!(
        matches!(error, DecodeError::UnsupportedDimensions { ndim: 2 }),
        "got: {error}"
    );
    assert!(error.to_string().contains("dimension"), "got: {error}");
}

#[test]
fn a_negative_dimension_count_is_rejected() {
    let body = [255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 23];
    let error = decode_field(oid::INT4_ARRAY, &body).expect_err("ndim -1 must be rejected");
    assert!(
        matches!(error, DecodeError::UnsupportedDimensions { .. }),
        "got: {error}"
    );
}

/// The header's element OID must agree with the column's array OID. A disagreement
/// means one of the two is wrong, and decoding either way is a guess.
#[test]
fn a_contradictory_element_oid_in_the_header_is_rejected() {
    let body = [
        0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 25, // header says text
        0, 0, 0, 0, 0, 0, 0, 1,
    ];
    let error = decode_field(oid::INT4_ARRAY, &body).expect_err("oid mismatch");
    assert!(error.to_string().contains("element OID"), "got: {error}");
}

#[test]
fn a_truncated_array_element_is_a_named_error() {
    let body = [
        0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 23, 0, 0, 0, 1, 0, 0, 0, 1, //
        0, 0, 0, 4, 0, 0, // claims 4 bytes, sends 2
    ];
    let error = decode_field(oid::INT4_ARRAY, &body).expect_err("short element");
    assert!(error.to_string().contains("array element"), "got: {error}");
}

// ===========================================================================
// Fallback: an unknown OID must not fail the query.
// ===========================================================================

/// The contract the integrator depends on. An OID with no binary decoder is
/// RECOVERABLE: re-read that column as text. If this ever became a hard failure,
/// adding a column of an exotic type to one table would break every route selecting
/// it, which is precisely the blast radius the fallback exists to prevent.
///
/// 600 is `point`, 3904 is `int4range`, 869 is `inet` — all text-only here.
#[test]
fn unsupported_oids_are_recoverable_and_wire_faults_are_not() {
    for unknown in [600u32, 3_904, 869, 1_000_000] {
        let error =
            decode_field(unknown, &[1, 2, 3, 4]).expect_err("no binary decoder should exist");
        assert!(
            matches!(error, DecodeError::UnsupportedOid { .. }),
            "OID {unknown}: got {error}"
        );
        assert!(
            error.needs_text_fallback(),
            "OID {unknown} must be recoverable via the text path"
        );
        assert!(error.to_string().contains("text"), "got: {error}");
        assert!(
            !supports(unknown),
            "supports() must agree with decode_field"
        );
    }
}

#[test]
fn supports_agrees_with_the_decoder_for_every_registered_oid() {
    let registered = [
        oid::BOOL,
        oid::BYTEA,
        oid::CHAR,
        oid::NAME,
        oid::INT8,
        oid::INT2,
        oid::INT4,
        oid::TEXT,
        oid::OID,
        oid::JSON,
        oid::XML,
        oid::FLOAT4,
        oid::FLOAT8,
        oid::BPCHAR,
        oid::VARCHAR,
        oid::DATE,
        oid::TIME,
        oid::TIMESTAMP,
        oid::TIMESTAMPTZ,
        oid::NUMERIC,
        oid::UUID,
        oid::JSONB,
    ];
    for type_oid in registered {
        assert!(supports(type_oid), "OID {type_oid} should be supported");
        // And it never reports UnsupportedOid, whatever the body.
        if let Err(error) = decode_field(type_oid, &[0; 8]) {
            assert!(!error.needs_text_fallback(), "OID {type_oid}: {error}");
        }
    }
    // Arrays of supported elements are supported too.
    for array_oid in [oid::INT4_ARRAY, oid::TEXT_ARRAY, oid::TIMESTAMPTZ_ARRAY] {
        assert!(supports(array_oid), "OID {array_oid} should be supported");
    }
}

#[test]
fn an_array_of_an_unsupported_element_type_also_falls_back() {
    // 1017 is point[]; element_of does not map it, so the array OID itself is unknown.
    let error = decode_field(1_017, &[0; 12]).expect_err("point[] has no binary decoder");
    assert!(error.needs_text_fallback(), "got: {error}");
}

// ===========================================================================
// Typed parameter binding.
// ===========================================================================

#[test]
fn integers_encode_big_endian_at_their_declared_width() {
    assert_eq!(encode(oid::INT2, &Value::Int(-3)), vec![255, 253]);
    assert_eq!(encode(oid::INT4, &Value::Int(66_051)), vec![0, 1, 2, 3]);
    assert_eq!(
        encode(oid::INT8, &Value::Int(7)),
        vec![0, 0, 0, 0, 0, 0, 0, 7]
    );
    assert_eq!(encode(oid::OID, &Value::Int(4_294_967_295)), vec![255; 4]);
}

/// Narrowing must be rejected, not truncated. 70000 into an int2 would wrap to 4464.
#[test]
fn an_out_of_range_integer_is_rejected_rather_than_wrapped() {
    let error = encode_param(oid::INT2, &Value::Int(70_000)).expect_err("70000 exceeds int2");
    assert!(error.to_string().contains("int2"), "got: {error}");
    assert!(error.to_string().contains("70000"), "got: {error}");
    // And a negative oid, which is unsigned.
    assert!(encode_param(oid::OID, &Value::Int(-1)).is_err());
}

#[test]
fn floats_and_bools_encode_to_their_wire_forms() {
    assert_eq!(
        encode(oid::FLOAT4, &Value::Float(1.5)),
        vec![0x3F, 0xC0, 0, 0]
    );
    assert_eq!(
        encode(oid::FLOAT8, &Value::Float(-2.25)),
        vec![0xC0, 0x02, 0, 0, 0, 0, 0, 0]
    );
    // An int widens into a float column, which is unambiguous.
    assert_eq!(
        encode(oid::FLOAT8, &Value::Int(1)),
        1.0f64.to_bits().to_be_bytes()
    );
    assert_eq!(encode(oid::BOOL, &Value::Bool(true)), vec![1]);
    assert_eq!(encode(oid::BOOL, &Value::Bool(false)), vec![0]);
}

#[test]
fn text_family_parameters_encode_as_utf8_and_jsonb_gets_its_version_byte() {
    assert_eq!(encode(oid::TEXT, &str_value("hi")), b"hi".to_vec());
    assert_eq!(encode(oid::JSON, &str_value("{}")), b"{}".to_vec());
    assert_eq!(encode(oid::JSONB, &str_value("{}")), vec![1, b'{', b'}']);
    // Round-trips back through the decoder, version byte and all.
    let bytes = encode(oid::JSONB, &str_value(r#"{"a":1}"#));
    assert_eq!(decode(oid::JSONB, &bytes), str_value(r#"{"a":1}"#));
}

#[test]
fn uuid_parameters_parse_to_16_bytes_and_reject_a_malformed_string() {
    let bytes = encode(
        oid::UUID,
        &str_value("00010203-0405-0607-0809-0a0b0c0d0e0f"),
    );
    assert_eq!(bytes, (0u8..16).collect::<Vec<u8>>());
    // Unhyphenated is accepted too.
    assert_eq!(
        encode(oid::UUID, &str_value("000102030405060708090a0b0c0d0e0f")),
        bytes
    );
    let error = encode_param(oid::UUID, &str_value("nope")).expect_err("not a uuid");
    assert!(error.to_string().contains("uuid"), "got: {error}");
}

/// NULL is None, and a zero-length value is Some(vec![]). The caller writes -1 for
/// the former and 0 for the latter; collapsing them corrupts both directions.
#[test]
fn nil_encodes_to_none_and_an_empty_string_to_an_empty_vec() {
    for type_oid in [oid::INT4, oid::TEXT, oid::TIMESTAMPTZ, oid::NUMERIC] {
        assert_eq!(encode_param(type_oid, &Value::Nil).unwrap(), None);
    }
    assert_eq!(
        encode_param(oid::TEXT, &str_value("")).unwrap(),
        Some(vec![])
    );
    assert_ne!(
        encode_param(oid::TEXT, &str_value("")).unwrap(),
        encode_param(oid::TEXT, &Value::Nil).unwrap()
    );
}

/// The epoch shift runs in reverse when encoding, and the round trip proves it.
#[test]
fn timestamp_parameters_apply_the_epoch_shift_in_reverse() {
    let bytes = encode(oid::TIMESTAMPTZ, &str_value("2024-01-15T10:30:00Z"));
    assert_eq!(bytes, 758_629_800_000_000i64.to_be_bytes().to_vec());
    // Round-trips exactly.
    assert_eq!(
        decode(oid::TIMESTAMPTZ, &bytes),
        str_value("2024-01-15T10:30:00Z")
    );
    // The 2000 epoch, not 1970.
    let zero = encode(oid::TIMESTAMPTZ, &str_value("2000-01-01T00:00:00Z"));
    assert_eq!(zero, 0i64.to_be_bytes().to_vec());
}

#[test]
fn date_and_time_parameters_round_trip_through_the_decoder() {
    let date = encode(oid::DATE, &str_value("2024-01-15"));
    assert_eq!(date, 8_780i32.to_be_bytes().to_vec());
    assert_eq!(decode(oid::DATE, &date), str_value("2024-01-15"));

    let time = encode(oid::TIME, &str_value("10:30:00.123456"));
    assert_eq!(time, 37_800_123_456i64.to_be_bytes().to_vec());
    assert_eq!(decode(oid::TIME, &time), str_value("10:30:00.123456"));
}

/// A fractional second must be right-padded: ".5" is 500000 µs, not 5 µs.
#[test]
fn a_fractional_second_is_right_padded_to_microseconds() {
    let time = encode(oid::TIME, &str_value("00:00:00.5"));
    assert_eq!(time, 500_000i64.to_be_bytes().to_vec());
    assert_ne!(time, 5i64.to_be_bytes().to_vec());
}

#[test]
fn malformed_date_and_time_strings_are_each_rejected_by_name() {
    let cases = [
        (oid::DATE, "not a date"),
        (oid::DATE, "2024-13-01"),
        (oid::DATE, "2024-01-32"),
        (oid::TIME, "25:00:00"),
        (oid::TIME, "10:60:00"),
        (oid::TIME, "10:30:60"),
        (oid::TIME, "10:30:00.1234567"),
        (oid::TIMESTAMPTZ, "2024-01-15"),
        (oid::TIMESTAMPTZ, "not a date"),
        (oid::TIMESTAMPTZ, "2024-13-01T00:00:00Z"),
        (oid::TIMESTAMPTZ, "2024-01-15T25:00:00"),
    ];
    for (type_oid, bad) in cases {
        let error = encode_param(type_oid, &str_value(bad))
            .expect_err(&format!("{bad:?} should be rejected for OID {type_oid}"));
        assert!(error.to_string().contains("parse"), "got: {error}");
    }
}

/// numeric must be bound from a string, because a float has already lost exactness.
#[test]
fn numeric_parameters_stay_exact_and_a_float_is_refused() {
    let bytes = encode(oid::NUMERIC, &str_value("19.99"));
    assert_eq!(numeric_to_string(&bytes).unwrap(), "19.99");
    // An int is exact and accepted.
    assert_eq!(
        numeric_to_string(&encode(oid::NUMERIC, &Value::Int(5))).unwrap(),
        "5"
    );
    // A float is refused, naming the exactness reason.
    let error = encode_param(oid::NUMERIC, &Value::Float(19.99)).expect_err("float refused");
    assert!(error.to_string().contains("exact"), "got: {error}");
}

#[test]
fn numeric_parameters_round_trip_for_zero_negative_nan_and_high_scale() {
    let cases = [
        "0",
        "0.00",
        "1",
        "-1",
        "19.99",
        "-0.50",
        "12345.6789",
        "7.0007",
        "0.000000000000000000000000000001",
        "123456789012345678901234567890",
        "NaN",
    ];
    for text in cases {
        let bytes = encode(oid::NUMERIC, &str_value(text));
        assert_eq!(
            numeric_to_string(&bytes).unwrap(),
            text,
            "{text:?} must survive encode then decode byte-exactly"
        );
    }
}

#[test]
fn a_malformed_numeric_string_is_rejected_by_name() {
    for bad in ["", "1.2.3", "12a", "1e5", "-"] {
        let error = encode_param(oid::NUMERIC, &str_value(bad))
            .expect_err(&format!("{bad:?} is not a decimal"));
        assert!(error.to_string().contains("numeric"), "got: {error}");
    }
}

#[test]
fn array_parameters_round_trip_including_a_null_element() {
    let value = list(vec![Value::Int(7), Value::Nil, Value::Int(-1)]);
    let bytes = encode(oid::INT4_ARRAY, &value);
    assert_eq!(decode(oid::INT4_ARRAY, &bytes), value);
    // has_null is set, and lower bound is 1 (not 0).
    assert_eq!(
        i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        1
    );
    assert_eq!(
        i32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        1
    );
}

#[test]
fn an_empty_list_encodes_as_a_zero_dimension_array() {
    let bytes = encode(oid::TEXT_ARRAY, &list(vec![]));
    assert_eq!(
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        0
    );
    assert_eq!(bytes.len(), 12, "no dimension block for an empty array");
    assert_eq!(decode(oid::TEXT_ARRAY, &bytes), list(vec![]));
}

#[test]
fn a_nested_list_is_rejected_rather_than_flattened() {
    let nested = list(vec![list(vec![Value::Int(1)])]);
    let error = encode_param(oid::INT4_ARRAY, &nested).expect_err("nesting is rejected");
    assert!(
        error.to_string().contains("multi-dimensional"),
        "got: {error}"
    );
}

#[test]
fn an_unbindable_value_names_both_the_type_and_the_value_kind() {
    let error = encode_param(oid::INT4, &str_value("hi")).expect_err("str is not an int4");
    assert!(error.to_string().contains("int4"), "got: {error}");
    assert!(error.to_string().contains("str"), "got: {error}");
}

#[test]
fn an_unsupported_parameter_oid_falls_back_to_text() {
    let error = encode_param(600, &Value::Int(1)).expect_err("point has no binary encoder");
    assert!(error.needs_text_fallback(), "got: {error}");
}

// ===========================================================================
// Bind format codes: how a caller actually requests binary.
// ===========================================================================

/// Count 0 means "everything is text" and no codes follow. That is what the current
/// `extended.rs` writes, which is why nothing is binary today.
#[test]
fn an_empty_format_code_array_means_all_text() {
    assert_eq!(format_codes(&[]), vec![0, 0]);
}

/// Count 1 means one code applies to every value — the compact form.
#[test]
fn a_uniform_format_code_array_collapses_to_the_compact_form() {
    assert_eq!(format_codes(&[FORMAT_BINARY]), vec![0, 1, 0, 1]);
    assert_eq!(
        format_codes(&[FORMAT_BINARY, FORMAT_BINARY, FORMAT_BINARY]),
        vec![0, 1, 0, 1],
        "three identical codes collapse to count 1"
    );
    assert_eq!(format_codes(&[FORMAT_TEXT, FORMAT_TEXT]), vec![0, 1, 0, 0]);
}

/// Count n means one code per value, and n must match the value count.
#[test]
fn a_mixed_format_code_array_is_spelled_out_in_full() {
    assert_eq!(
        format_codes(&[FORMAT_BINARY, FORMAT_TEXT]),
        vec![0, 2, 0, 1, 0, 0]
    );
    assert_eq!(
        format_codes(&[FORMAT_TEXT, FORMAT_BINARY, FORMAT_TEXT]),
        vec![0, 3, 0, 0, 0, 1, 0, 0]
    );
}

#[test]
fn the_format_code_constants_match_the_protocol() {
    assert_eq!(FORMAT_TEXT, 0);
    assert_eq!(FORMAT_BINARY, 1);
}
