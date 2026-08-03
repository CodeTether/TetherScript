//! Integration tests for the RESP wire-protocol codec.
//!
//! Every reply is written out as literal bytes rather than round-tripped through
//! the encoder, because a codec that only agrees with itself proves nothing. The
//! byte strings here are the ones a real Redis server puts on the wire.

use tetherscript::resp::codec::limits::{MAX_AGGREGATE_LEN, MAX_BULK_LEN, MAX_DEPTH};
use tetherscript::resp::codec::{decode, encode_command, DecodeError, Reply};

/// Decode a buffer expected to hold exactly one complete reply, asserting that
/// the reply accounts for every byte.
fn one(bytes: &[u8]) -> Reply {
    let (reply, consumed) = decode(bytes).expect("reply should decode");
    assert_eq!(consumed, bytes.len(), "should consume the whole buffer");
    reply
}

/// Assert the buffer is rejected as malformed, returning the message so the test
/// can check that it names the problem.
fn malformed(bytes: &[u8]) -> String {
    match decode(bytes) {
        Err(DecodeError::Malformed(message)) => message,
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn decodes_simple_string() {
    assert_eq!(one(b"+OK\r\n"), Reply::Simple("OK".into()));
    assert_eq!(one(b"+\r\n"), Reply::Simple(String::new()));
}

#[test]
fn decodes_error_as_a_value_not_a_transport_failure() {
    let reply = one(b"-WRONGTYPE Operation against a key\r\n");
    assert_eq!(
        reply,
        Reply::Error("WRONGTYPE Operation against a key".into())
    );
    assert_eq!(reply.error_code(), Some("WRONGTYPE"));
}

#[test]
fn decodes_blob_error_as_the_same_error_variant() {
    assert_eq!(
        one(b"!21\r\nSYNTAX invalid syntax\r\n"),
        Reply::Error("SYNTAX invalid syntax".into())
    );
}

#[test]
fn decodes_integer_including_negatives_and_bounds() {
    assert_eq!(one(b":0\r\n"), Reply::Integer(0));
    assert_eq!(one(b":-42\r\n"), Reply::Integer(-42));
    assert_eq!(one(b":9223372036854775807\r\n"), Reply::Integer(i64::MAX));
    assert!(malformed(b":nope\r\n").contains("invalid integer"));
    // One past i64::MAX must be rejected rather than silently saturated.
    assert!(malformed(b":9223372036854775808\r\n").contains("invalid integer"));
}

#[test]
fn decodes_bulk_string() {
    assert_eq!(one(b"$5\r\nhello\r\n"), Reply::Bulk(b"hello".to_vec()));
}

#[test]
fn null_bulk_string_is_not_an_empty_bulk_string() {
    let missing = one(b"$-1\r\n");
    let empty = one(b"$0\r\n\r\n");
    assert_eq!(missing, Reply::Nil);
    assert_eq!(empty, Reply::Bulk(Vec::new()));
    assert_ne!(missing, empty, "a cache miss is not a cached empty value");
    assert!(missing.is_nil());
    assert!(!empty.is_nil());
    assert_eq!(missing.as_bytes(), None);
    assert_eq!(empty.as_bytes(), Some(&b""[..]));
}

#[test]
fn resp3_null_decodes_to_the_same_nil() {
    assert_eq!(one(b"_\r\n"), Reply::Nil);
    assert!(malformed(b"_x\r\n").contains("null must be"));
}

#[test]
fn bulk_length_is_a_byte_count_so_crlf_inside_the_payload_is_data() {
    assert_eq!(one(b"$4\r\na\r\nb\r\n"), Reply::Bulk(b"a\r\nb".to_vec()));
    // A payload that is CRLF and nothing else.
    assert_eq!(one(b"$2\r\n\r\n\r\n"), Reply::Bulk(b"\r\n".to_vec()));
}

#[test]
fn bulk_payload_need_not_be_valid_utf8() {
    let reply = one(b"$3\r\n\xff\xfe\x00\r\n");
    assert_eq!(reply, Reply::Bulk(vec![0xff, 0xfe, 0x00]));
    assert!(String::from_utf8(vec![0xff, 0xfe, 0x00]).is_err());
}

#[test]
fn bulk_payload_length_must_match_the_framing() {
    // Announces three bytes, but the CRLF does not land where it must.
    assert!(malformed(b"$3\r\nab\r\nxx").contains("expected CRLF"));
}

#[test]
fn decodes_array_and_distinguishes_null_from_empty() {
    assert_eq!(
        one(b"*2\r\n$3\r\nfoo\r\n:7\r\n"),
        Reply::Array(vec![Reply::Bulk(b"foo".to_vec()), Reply::Integer(7)])
    );
    let missing = one(b"*-1\r\n");
    let empty = one(b"*0\r\n");
    assert_eq!(missing, Reply::Nil);
    assert_eq!(empty, Reply::Array(Vec::new()));
    assert_ne!(missing, empty);
}

#[test]
fn decodes_nested_arrays_of_arrays() {
    let bytes = b"*2\r\n*2\r\n:1\r\n:2\r\n*1\r\n*1\r\n$1\r\nx\r\n";
    assert_eq!(
        one(bytes),
        Reply::Array(vec![
            Reply::Array(vec![Reply::Integer(1), Reply::Integer(2)]),
            Reply::Array(vec![Reply::Array(vec![Reply::Bulk(b"x".to_vec())])]),
        ])
    );
}

#[test]
fn decodes_map_in_wire_order() {
    let bytes = b"%2\r\n$5\r\nproto\r\n:3\r\n$2\r\nid\r\n:12\r\n";
    assert_eq!(
        one(bytes),
        Reply::Map(vec![
            (Reply::Bulk(b"proto".to_vec()), Reply::Integer(3)),
            (Reply::Bulk(b"id".to_vec()), Reply::Integer(12)),
        ])
    );
    assert_eq!(one(b"%0\r\n"), Reply::Map(Vec::new()));
}

#[test]
fn decodes_set() {
    assert_eq!(
        one(b"~2\r\n+a\r\n+b\r\n"),
        Reply::Set(vec![Reply::Simple("a".into()), Reply::Simple("b".into())])
    );
}

#[test]
fn decodes_push_distinctly_from_array() {
    let bytes = b">3\r\n$7\r\nmessage\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
    let expected = vec![
        Reply::Bulk(b"message".to_vec()),
        Reply::Bulk(b"foo".to_vec()),
        Reply::Bulk(b"bar".to_vec()),
    ];
    assert_eq!(one(bytes), Reply::Push(expected.clone()));
    assert_ne!(one(bytes), Reply::Array(expected));
}

#[test]
fn decodes_double_including_the_special_forms() {
    assert_eq!(one(b",3.5\r\n"), Reply::Double(3.5));
    assert_eq!(one(b",10\r\n"), Reply::Double(10.0));
    assert_eq!(one(b",inf\r\n"), Reply::Double(f64::INFINITY));
    assert_eq!(one(b",-inf\r\n"), Reply::Double(f64::NEG_INFINITY));
    match one(b",nan\r\n") {
        Reply::Double(value) => assert!(value.is_nan()),
        other => panic!("expected a double, got {other:?}"),
    }
    assert!(malformed(b",abc\r\n").contains("double has invalid value"));
}

#[test]
fn decodes_boolean() {
    assert_eq!(one(b"#t\r\n"), Reply::Boolean(true));
    assert_eq!(one(b"#f\r\n"), Reply::Boolean(false));
    assert!(malformed(b"#true\r\n").contains("boolean must be"));
}

#[test]
fn decodes_big_number_as_text() {
    let digits = "3492890328409238509324850943850943825024385";
    assert_eq!(
        one(format!("({digits}\r\n").as_bytes()),
        Reply::BigNumber(digits.into())
    );
    assert_eq!(one(b"(-12\r\n"), Reply::BigNumber("-12".into()));
    assert!(malformed(b"(12x\r\n").contains("big number has invalid value"));
    assert!(malformed(b"(-\r\n").contains("big number has invalid value"));
}

#[test]
fn decodes_verbatim_string_and_splits_the_format_hint() {
    assert_eq!(
        one(b"=15\r\ntxt:Some string\r\n"),
        Reply::Verbatim {
            format: "txt".into(),
            text: b"Some string".to_vec(),
        }
    );
    assert!(malformed(b"=3\r\ntxt\r\n").contains("three-byte format hint"));
    assert!(malformed(b"=-1\r\n").contains("no null form"));
}

#[test]
fn unknown_type_byte_is_malformed_not_incomplete() {
    assert!(malformed(b"^nope\r\n").contains("unknown type byte"));
}

/// Every proper prefix of a complete reply must be `Incomplete`. This is the
/// property a socket read loop depends on: a short read is not corruption.
#[test]
fn every_prefix_of_every_reply_type_is_incomplete() {
    let replies: &[&[u8]] = &[
        b"+OK\r\n",
        b"-ERR bad\r\n",
        b":1234\r\n",
        b"$5\r\nhello\r\n",
        b"$-1\r\n",
        b"$0\r\n\r\n",
        b"$4\r\na\r\nb\r\n",
        b"_\r\n",
        b",3.5\r\n",
        b"#t\r\n",
        b"(123456789012345678901234567890\r\n",
        b"=15\r\ntxt:Some string\r\n",
        b"!8\r\nERR oops\r\n",
        b"*2\r\n$3\r\nfoo\r\n:7\r\n",
        b"*2\r\n*2\r\n:1\r\n:2\r\n*1\r\n*1\r\n$1\r\nx\r\n",
        b"%2\r\n$5\r\nproto\r\n:3\r\n$2\r\nid\r\n:12\r\n",
        b"~2\r\n+a\r\n+b\r\n",
        b">2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n",
    ];
    for reply in replies {
        for split in 0..reply.len() {
            assert_eq!(
                decode(&reply[..split]),
                Err(DecodeError::Incomplete),
                "prefix of {split} byte(s) of {reply:?} should be Incomplete"
            );
        }
        decode(reply).expect("the full reply still decodes");
    }
}

#[test]
fn hostile_bulk_length_is_rejected_not_allocated() {
    let message = malformed(b"$9999999999\r\n");
    assert!(message.contains("9999999999"));
    assert!(message.contains(&MAX_BULK_LEN.to_string()));
    // Exactly one byte past the documented bound.
    let over = format!("${}\r\n", MAX_BULK_LEN + 1).into_bytes();
    assert!(malformed(&over).contains("exceeds"));
    assert!(malformed(b"$-2\r\n").contains("negative length"));
}

#[test]
fn hostile_aggregate_count_is_rejected() {
    for prefix in ["*", "~", "%", ">"] {
        let bytes = format!("{prefix}{}\r\n", MAX_AGGREGATE_LEN + 1).into_bytes();
        assert!(
            malformed(&bytes).contains("exceeds"),
            "a {prefix} count past the bound should be rejected"
        );
    }
    assert!(malformed(b"*-2\r\n").contains("negative length"));
}

#[test]
fn nesting_at_the_depth_limit_is_accepted_and_past_it_is_rejected() {
    let at_limit = format!("{}:1\r\n", "*1\r\n".repeat(MAX_DEPTH)).into_bytes();
    decode(&at_limit).expect("nesting exactly at the limit is legal");

    let past_limit = format!("{}:1\r\n", "*1\r\n".repeat(MAX_DEPTH + 1)).into_bytes();
    assert!(malformed(&past_limit).contains("nests deeper"));
}

#[test]
fn an_unterminated_line_is_bounded_rather_than_incomplete_forever() {
    let mut bytes = vec![b'+'];
    bytes.extend(std::iter::repeat_n(b'a', 70 * 1024));
    assert!(malformed(&bytes).contains("no CRLF"));
}

#[test]
fn trailing_bytes_are_left_for_the_next_reply() {
    let buf: &[u8] = b"+OK\r\n$3\r\nfoo\r\n:9\r\n";
    let (first, used) = decode(buf).expect("first reply decodes");
    assert_eq!(first, Reply::Simple("OK".into()));
    assert_eq!(used, 5);

    let (second, used_second) = decode(&buf[used..]).expect("second reply decodes");
    assert_eq!(second, Reply::Bulk(b"foo".to_vec()));
    assert_eq!(used_second, 9);

    let (third, used_third) = decode(&buf[used + used_second..]).expect("third reply decodes");
    assert_eq!(third, Reply::Integer(9));
    assert_eq!(used_third, 4);
    assert_eq!(used + used_second + used_third, buf.len());
}

#[test]
fn encodes_a_command_as_an_array_of_bulk_strings() {
    assert_eq!(
        encode_command(&[b"SET".as_slice(), b"key", b"value"]),
        b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n".to_vec()
    );
    assert_eq!(
        encode_command(&[b"PING".as_slice()]),
        b"*1\r\n$4\r\nPING\r\n".to_vec()
    );
}

#[test]
fn encoded_arguments_are_binary_safe() {
    assert_eq!(
        encode_command(&[b"SET".as_slice(), b"k", b"a\r\nb"]),
        b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$4\r\na\r\nb\r\n".to_vec()
    );
    assert_eq!(
        encode_command(&[b"SET".as_slice(), b"k", b""]),
        b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$0\r\n\r\n".to_vec()
    );
}

#[test]
#[should_panic(expected = "needs at least a name")]
fn encoding_an_empty_command_panics() {
    encode_command(&[]);
}
