//! RESP codec tests. No server required.
//!
//! These cover the framing decisions that are easy to get wrong and impossible to
//! notice until production: CRLF inside a bulk body, the null bulk string versus
//! the empty one, a truncated reply, an oversized declared length, and an error
//! reply surfaced as a named failure rather than a transport fault.
//!
//! Only the public surface of the client is used, so these tests also pin the API
//! the integrator wires up.

use tetherscript::redis::{decode, encode_command, Decoded, RedisError, RespValue, Ttl};

/// Unwrap a complete frame, failing loudly if the decoder wanted more bytes.
fn frame(input: &[u8]) -> (RespValue, usize) {
    match decode(input).expect("input should decode") {
        Decoded::Frame { value, consumed } => (value, consumed),
        Decoded::Incomplete => panic!("expected a complete frame from {input:?}"),
    }
}

/// Decode expecting a protocol rejection, returning its message.
fn protocol_error(input: &[u8]) -> String {
    match decode(input) {
        Err(RedisError::Protocol(message)) => message,
        other => panic!("expected a protocol error, got {other:?}"),
    }
}

// ---------------------------------------------------------------- encoding

/// A command must be a RESP array of bulk strings, byte for byte.
#[test]
fn encodes_command_as_array_of_bulk_strings() {
    let bytes = encode_command(&[&b"SET"[..], &b"key"[..], &b"value"[..]]).unwrap();
    assert_eq!(
        bytes,
        b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n".to_vec()
    );
}

/// PING has no arguments but is still a one-element array, not an inline line.
#[test]
fn encodes_argumentless_command_as_single_element_array() {
    assert_eq!(
        encode_command(&[&b"PING"[..]]).unwrap(),
        b"*1\r\n$4\r\nPING\r\n".to_vec()
    );
}

/// The injection case. A value containing CRLF must stay one length-counted
/// argument; the inline command form is what would turn it into a second command.
#[test]
fn encodes_crlf_in_a_value_as_data_not_structure() {
    let bytes = encode_command(&[&b"SET"[..], &b"k"[..], &b"a\r\nFLUSHALL\r\n"[..]]).unwrap();
    assert_eq!(
        bytes,
        b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$13\r\na\r\nFLUSHALL\r\n\r\n".to_vec()
    );
    // Round-tripping the payload through the decoder proves the length, not the
    // delimiter, defines the boundary.
    let (value, _) = frame(b"$13\r\na\r\nFLUSHALL\r\n\r\n");
    assert_eq!(value, RespValue::Bulk(b"a\r\nFLUSHALL\r\n".to_vec()));
}

/// Spaces in an argument would shift every later argument in the inline form.
#[test]
fn encodes_spaces_in_a_value_as_one_argument() {
    let bytes = encode_command(&[&b"SET"[..], &b"k"[..], &b"two words"[..]]).unwrap();
    assert_eq!(
        bytes,
        b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$9\r\ntwo words\r\n".to_vec()
    );
}

/// An empty argument is legal and encodes as a zero-length bulk string.
#[test]
fn encodes_empty_argument_as_zero_length_bulk() {
    let bytes = encode_command(&[&b"SET"[..], &b"k"[..], &b""[..]]).unwrap();
    assert_eq!(bytes, b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$0\r\n\r\n".to_vec());
}

/// Non-UTF-8 bytes survive, because Redis values are binary-safe.
#[test]
fn encodes_binary_argument_verbatim() {
    let payload = [0x00u8, 0xff, 0x0d, 0x0a, 0x80];
    let bytes = encode_command(&[&b"SET"[..], &b"k"[..], &payload[..]]).unwrap();
    assert!(
        bytes.ends_with(&[b'$', b'5', b'\r', b'\n', 0x00, 0xff, 0x0d, 0x0a, 0x80, b'\r', b'\n'])
    );
}

/// A command with no name is a caller bug and must not reach the socket.
#[test]
fn rejects_empty_command() {
    assert!(matches!(
        encode_command(&[]).unwrap_err(),
        RedisError::Protocol(_)
    ));
}

// ------------------------------------------------------- decoding each type

/// `+OK\r\n`.
#[test]
fn decodes_simple_string() {
    let (value, consumed) = frame(b"+OK\r\n");
    assert_eq!(value, RespValue::Simple("OK".into()));
    assert_eq!(consumed, 5);
    assert_eq!(value.simple("PING").unwrap(), "OK");
}

/// An empty status line is still a status line.
#[test]
fn decodes_empty_simple_string() {
    let (value, _) = frame(b"+\r\n");
    assert_eq!(value, RespValue::Simple(String::new()));
}

/// `:42\r\n`, including negative values, which `TTL` relies on.
#[test]
fn decodes_integer() {
    assert_eq!(frame(b":42\r\n").0, RespValue::Integer(42));
    assert_eq!(frame(b":0\r\n").0, RespValue::Integer(0));
    assert_eq!(frame(b":-2\r\n").0, RespValue::Integer(-2));
}

/// The full i64 range must round-trip; Redis counters can reach it.
#[test]
fn decodes_extreme_integers() {
    let max = format!(":{}\r\n", i64::MAX);
    assert_eq!(frame(max.as_bytes()).0, RespValue::Integer(i64::MAX));
    let min = format!(":{}\r\n", i64::MIN);
    assert_eq!(frame(min.as_bytes()).0, RespValue::Integer(i64::MIN));
}

/// A non-numeric integer line is a framing failure, not a value.
#[test]
fn rejects_malformed_integer() {
    assert!(protocol_error(b":abc\r\n").contains("abc"));
}

/// `$5\r\nhello\r\n`.
#[test]
fn decodes_bulk_string() {
    let (value, consumed) = frame(b"$5\r\nhello\r\n");
    assert_eq!(value, RespValue::Bulk(b"hello".to_vec()));
    assert_eq!(consumed, 11);
    assert_eq!(value.bulk("GET").unwrap(), b"hello".as_slice());
}

/// The classic bug: a bulk body containing CRLF must be read by length. Splitting
/// on CRLF would yield `a` and desynchronise the connection from `bcd` onward.
#[test]
fn decodes_bulk_string_containing_crlf() {
    // 6 declared bytes: `a`, CR, LF, `b`, `c`, `d`. The trailing CRLF is the
    // terminator, not part of the payload.
    let (value, consumed) = frame(b"$6\r\na\r\nbcd\r\n");
    assert_eq!(value, RespValue::Bulk(b"a\r\nbcd".to_vec()));
    assert_eq!(consumed, 12);
}

/// A body that is nothing but CRLFs is still exactly `len` bytes.
#[test]
fn decodes_bulk_string_of_only_crlf() {
    let (value, _) = frame(b"$4\r\n\r\n\r\n\r\n");
    assert_eq!(value, RespValue::Bulk(b"\r\n\r\n".to_vec()));
}

/// Arbitrary bytes, including NUL and invalid UTF-8, survive decoding.
#[test]
fn decodes_binary_bulk_string() {
    let input = [b'$', b'3', b'\r', b'\n', 0x00, 0xff, 0x80, b'\r', b'\n'];
    assert_eq!(frame(&input).0, RespValue::Bulk(vec![0x00, 0xff, 0x80]));
}

/// A bulk body must be followed by CRLF; anything else means the length lied.
#[test]
fn rejects_bulk_string_without_trailing_crlf() {
    assert!(protocol_error(b"$3\r\nabcXX").contains("CRLF"));
}

// ----------------------------------------------- null bulk versus empty bulk

/// `$-1\r\n` is *absent*; `$0\r\n\r\n` is *present and empty*. A session store that
/// conflates them cannot tell a logged-out user from an empty session value.
#[test]
fn distinguishes_null_bulk_from_empty_bulk() {
    let (null, null_consumed) = frame(b"$-1\r\n");
    let (empty, empty_consumed) = frame(b"$0\r\n\r\n");

    assert_eq!(null, RespValue::NullBulk);
    assert_eq!(empty, RespValue::Bulk(Vec::new()));
    assert_ne!(null, empty);
    assert_eq!(null_consumed, 5);
    assert_eq!(empty_consumed, 6);

    assert!(null.is_null());
    assert!(!empty.is_null());
    assert_eq!(null.type_name(), "null-bulk");
    assert_eq!(empty.type_name(), "bulk");

    // And the typed accessors keep the distinction.
    assert_eq!(null.optional_bulk("GET").unwrap(), None);
    assert_eq!(empty.optional_bulk("GET").unwrap(), Some(b"".as_slice()));
    assert!(null.bulk("GET").is_err());
    assert_eq!(empty.bulk("GET").unwrap(), b"".as_slice());
}

/// Only `-1` means null. Any other negative length is malformed, so a corrupt
/// `$-7` is reported rather than silently treated as absent.
#[test]
fn rejects_negative_bulk_length_other_than_minus_one() {
    let message = protocol_error(b"$-7\r\n");
    assert!(message.contains("-7"));
    assert!(message.contains("only -1 means null"));
}

// ------------------------------------------------------------------ arrays

/// `*2\r\n` with two bulk elements.
#[test]
fn decodes_array() {
    let (value, consumed) = frame(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    assert_eq!(
        value,
        RespValue::Array(vec![
            RespValue::Bulk(b"foo".to_vec()),
            RespValue::Bulk(b"bar".to_vec()),
        ])
    );
    assert_eq!(consumed, 22);
}

/// A nested array, with mixed element types at both levels.
#[test]
fn decodes_nested_array() {
    let input = b"*3\r\n:1\r\n*2\r\n$3\r\ntwo\r\n:3\r\n+OK\r\n";
    let (value, consumed) = frame(input);
    assert_eq!(
        value,
        RespValue::Array(vec![
            RespValue::Integer(1),
            RespValue::Array(vec![
                RespValue::Bulk(b"two".to_vec()),
                RespValue::Integer(3),
            ]),
            RespValue::Simple("OK".into()),
        ])
    );
    assert_eq!(consumed, input.len());
}

/// Three levels deep, to prove recursion carries the offset correctly.
#[test]
fn decodes_deeply_nested_array() {
    let (value, _) = frame(b"*1\r\n*1\r\n*1\r\n:9\r\n");
    assert_eq!(
        value,
        RespValue::Array(vec![RespValue::Array(vec![RespValue::Array(vec![
            RespValue::Integer(9)
        ])])])
    );
}

/// Null elements inside an array keep their identity.
#[test]
fn decodes_array_with_null_element() {
    let (value, _) = frame(b"*2\r\n$-1\r\n$0\r\n\r\n");
    assert_eq!(
        value,
        RespValue::Array(vec![RespValue::NullBulk, RespValue::Bulk(Vec::new())])
    );
}

/// `*-1\r\n` is the null array; `*0\r\n` is an empty one. Distinct, as with bulk.
#[test]
fn distinguishes_null_array_from_empty_array() {
    let (null, null_consumed) = frame(b"*-1\r\n");
    let (empty, empty_consumed) = frame(b"*0\r\n");

    assert_eq!(null, RespValue::NullArray);
    assert_eq!(empty, RespValue::Array(Vec::new()));
    assert_ne!(null, empty);
    assert_eq!(null_consumed, 5);
    assert_eq!(empty_consumed, 4);
    assert!(null.is_null());
    assert!(!empty.is_null());
}

/// Only `-1` means null for arrays too.
#[test]
fn rejects_negative_array_length_other_than_minus_one() {
    assert!(protocol_error(b"*-3\r\n").contains("only -1 means null"));
}

// ------------------------------------------------------ incremental safety

/// Every prefix of a valid reply must report *incomplete*, never panic and never
/// decode a wrong value. This is the property a socket read loop depends on.
#[test]
fn every_prefix_of_a_reply_is_incomplete_not_a_panic() {
    let complete = b"*2\r\n$3\r\nfoo\r\n$6\r\na\r\nbcd\r\n";
    for cut in 0..complete.len() {
        let partial = &complete[..cut];
        match decode(partial) {
            Ok(Decoded::Incomplete) => {}
            other => panic!("prefix of {cut} bytes should be incomplete, got {other:?}"),
        }
    }
    assert_eq!(frame(complete).1, complete.len());
}

/// A truncated bulk body reports need-more-bytes rather than returning the short
/// payload it happens to hold.
#[test]
fn truncated_bulk_string_reports_incomplete() {
    assert_eq!(decode(b"$5\r\nhel").unwrap(), Decoded::Incomplete);
    assert_eq!(decode(b"$5\r\nhello").unwrap(), Decoded::Incomplete);
    assert_eq!(decode(b"$5\r\nhello\r").unwrap(), Decoded::Incomplete);
    assert_eq!(
        frame(b"$5\r\nhello\r\n").0,
        RespValue::Bulk(b"hello".to_vec())
    );
}

/// A length line with no terminator yet is incomplete, not malformed.
#[test]
fn truncated_length_line_reports_incomplete() {
    assert_eq!(decode(b"$").unwrap(), Decoded::Incomplete);
    assert_eq!(decode(b"$1").unwrap(), Decoded::Incomplete);
    assert_eq!(decode(b"$12\r").unwrap(), Decoded::Incomplete);
}

/// An empty buffer is the most common short read of all.
#[test]
fn empty_input_reports_incomplete() {
    assert_eq!(decode(b"").unwrap(), Decoded::Incomplete);
}

/// A partial element makes the whole array incomplete, not a short array.
#[test]
fn truncated_array_element_reports_incomplete() {
    assert_eq!(
        decode(b"*2\r\n$3\r\nfoo\r\n$3\r\nba").unwrap(),
        Decoded::Incomplete
    );
}

/// A missing inner element makes the outer array incomplete too.
#[test]
fn truncated_nested_array_reports_incomplete() {
    assert_eq!(decode(b"*2\r\n*2\r\n:1\r\n").unwrap(), Decoded::Incomplete);
}

/// Pipelined replies: only the first frame's bytes are consumed, so a caller can
/// drain exactly `consumed` and decode the rest on the next call.
#[test]
fn consumes_only_the_first_of_several_replies() {
    let input = b"+OK\r\n:7\r\n$2\r\nhi\r\n";
    let (first, consumed) = frame(input);
    assert_eq!(first, RespValue::Simple("OK".into()));
    assert_eq!(consumed, 5);

    let (second, consumed_second) = frame(&input[consumed..]);
    assert_eq!(second, RespValue::Integer(7));

    let (third, _) = frame(&input[consumed + consumed_second..]);
    assert_eq!(third, RespValue::Bulk(b"hi".to_vec()));
}

// ------------------------------------------------------- hostile declarations

/// A hostile bulk length must be refused before anything is allocated.
#[test]
fn rejects_oversized_bulk_length() {
    let message = protocol_error(b"$536870913\r\n");
    assert!(message.contains("536870913"));
    assert!(message.contains("limit"));
}

/// Even an absurd length far past `usize` range is a clean rejection, not an
/// overflow or an allocation attempt.
#[test]
fn rejects_astronomical_bulk_length() {
    let input = format!("${}\r\n", i64::MAX);
    assert!(protocol_error(input.as_bytes()).contains("limit"));
}

/// A length beyond even `i64` is a parse failure, still reported cleanly.
#[test]
fn rejects_unparsable_bulk_length() {
    assert!(protocol_error(b"$99999999999999999999\r\n").contains("not a valid integer"));
}

/// An oversized array count is refused before the decode loop runs.
#[test]
fn rejects_oversized_array_length() {
    let message = protocol_error(b"*1048577\r\n");
    assert!(message.contains("1048577"));
    assert!(message.contains("limit"));
}

/// Unbounded nesting must not recurse until the stack overflows.
#[test]
fn rejects_excessively_nested_array() {
    let mut input = Vec::new();
    for _ in 0..64 {
        input.extend_from_slice(b"*1\r\n");
    }
    input.extend_from_slice(b":1\r\n");
    assert!(protocol_error(&input).contains("nests deeper"));
}

/// An unterminated control line cannot grow the buffer forever.
#[test]
fn rejects_unterminated_control_line() {
    let mut input = vec![b'+'];
    input.extend(std::iter::repeat_n(b'x', 64 * 1024 + 1));
    assert!(protocol_error(&input).contains("unterminated line"));
}

/// An unknown leading byte is a framing failure naming the byte.
#[test]
fn rejects_unknown_type_byte() {
    assert!(protocol_error(b"?what\r\n").contains("0x3f"));
}

// --------------------------------------------------------------- error replies

/// An error reply decodes successfully into a named failure. The transport
/// worked, so it must not look like a socket fault.
#[test]
fn decodes_error_reply_as_a_named_failure() {
    let input = b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n";
    let (value, consumed) = frame(input);
    assert_eq!(consumed, input.len());
    assert_eq!(
        value,
        RespValue::Error {
            kind: "WRONGTYPE".into(),
            message: "Operation against a key holding the wrong kind of value".into(),
        }
    );
    assert_eq!(value.type_name(), "error");
}

/// The plain `ERR` case, and an error line with no message at all.
#[test]
fn decodes_error_reply_variants() {
    assert_eq!(
        frame(b"-ERR unknown command 'nope'\r\n").0,
        RespValue::Error {
            kind: "ERR".into(),
            message: "unknown command 'nope'".into(),
        }
    );
    // No space: the whole line is the kind, and nothing is dropped.
    assert_eq!(
        frame(b"-NOAUTH\r\n").0,
        RespValue::Error {
            kind: "NOAUTH".into(),
            message: String::new(),
        }
    );
}

/// An error nested inside an array stays a value. This is why promotion to
/// `RedisError::Server` happens at the command layer, not in the decoder.
#[test]
fn keeps_nested_error_reply_as_a_value() {
    let (value, _) = frame(b"*2\r\n+OK\r\n-ERR partial failure\r\n");
    assert_eq!(
        value,
        RespValue::Array(vec![
            RespValue::Simple("OK".into()),
            RespValue::Error {
                kind: "ERR".into(),
                message: "partial failure".into(),
            },
        ])
    );
}

/// A server error is a protocol-level success; a transport error is not. The two
/// must stay tellable apart, including their kinds and their messages.
#[test]
fn separates_server_errors_from_transport_errors() {
    let server = RedisError::Server {
        kind: "WRONGTYPE".into(),
        message: "wrong kind of value".into(),
    };
    assert!(server.is_server_error());
    assert_eq!(server.kind(), Some("WRONGTYPE"));
    assert_eq!(
        server.to_string(),
        "redis: server: WRONGTYPE wrong kind of value"
    );

    let transport = RedisError::Transport("connection reset".into());
    assert!(!transport.is_server_error());
    assert_eq!(transport.kind(), None);
    assert_eq!(transport.to_string(), "redis: transport: connection reset");

    let protocol = RedisError::Protocol("bad length".into());
    assert!(!protocol.is_server_error());
    assert_eq!(protocol.to_string(), "redis: protocol: bad length");

    assert_ne!(server, transport);
}

/// A reply of the wrong shape is named, not panicked on.
#[test]
fn reports_unexpected_reply_type_by_name() {
    let error = RespValue::Integer(1).bulk("GET").unwrap_err();
    match error {
        RedisError::UnexpectedType(message) => {
            assert!(message.contains("GET"));
            assert!(message.contains("integer"));
        }
        other => panic!("expected an unexpected-type error, got {other:?}"),
    }
    assert!(RespValue::NullBulk.integer("TTL").is_err());
    assert!(RespValue::Integer(1).simple("SET").is_err());
}

// ------------------------------------------------------------------ TTL model

/// `TTL`'s negative sentinels are three distinct states, not one "expired".
#[test]
fn interprets_ttl_sentinels() {
    assert_eq!(Ttl::from_reply(60), Ttl::Seconds(60));
    assert_eq!(Ttl::from_reply(0), Ttl::Seconds(0));
    assert_eq!(Ttl::from_reply(-1), Ttl::Persistent);
    assert_eq!(Ttl::from_reply(-2), Ttl::Missing);
    // Any other negative is treated as missing rather than as a lifetime.
    assert_eq!(Ttl::from_reply(-99), Ttl::Missing);
    assert_ne!(Ttl::Persistent, Ttl::Missing);
}
