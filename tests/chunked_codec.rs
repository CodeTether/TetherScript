//! Integration tests for the HTTP/1.1 chunked transfer-encoding codec.
//!
//! These exercise the public surface of `tetherscript::chunked::codec` only — the codec is
//! deliberately independent of the HTTP server, which another change wires up.
//!
//! The tests are grouped by concern: round-tripping, incremental decoding, chunk
//! extensions, trailers, the strict size grammar, the hard bounds, and the streaming
//! response head.

use tetherscript::chunked::codec::{
    decode, decode_trailers, encode_body, encode_chunk, encode_last_chunk, parse_chunk_size,
    streaming_head, strip_extensions, ChunkedError, MAX_CHUNK_BYTES, MAX_TRAILERS,
};

/// A `Malformed` outcome, with a helpful message when the value is anything else.
fn assert_malformed(result: Result<impl std::fmt::Debug, ChunkedError>, what: &str) {
    match result {
        Err(ChunkedError::Malformed(_)) => {}
        other => panic!("{what} should be Malformed, got {other:?}"),
    }
}

// ---------------------------------------------------------------- round trip

#[test]
fn encode_then_decode_round_trips() {
    let payloads: [&[u8]; 4] = [b"", b"x", b"hello world", &[0xff; 4096]];
    for payload in payloads {
        let wire = encode_body(payload, &[]).expect("encode");
        let body = decode(&wire).expect("decode");
        assert_eq!(body.payload, payload, "payload survives the round trip");
        assert_eq!(body.consumed, wire.len(), "whole body consumed");
        assert!(body.trailers.is_empty());
    }
}

#[test]
fn single_chunk_has_the_documented_wire_form() {
    assert_eq!(encode_chunk(b"hello").unwrap(), b"5\r\nhello\r\n".to_vec());
    assert_eq!(encode_chunk(&[0u8; 16]).unwrap()[..4], b"10\r\n"[..]);
}

#[test]
fn empty_body_encodes_to_just_the_zero_chunk() {
    assert_eq!(encode_body(b"", &[]).unwrap(), b"0\r\n\r\n".to_vec());
    // An empty slice must never be encoded as a data chunk, since `0\r\n` is the terminator.
    assert!(encode_chunk(b"").unwrap().is_empty());

    let body = decode(b"0\r\n\r\n").unwrap();
    assert!(body.payload.is_empty());
    assert!(body.trailers.is_empty());
    assert_eq!(body.consumed, 5);
}

#[test]
fn multi_chunk_body_concatenates_in_order() {
    // 0xd == 13 == the length of " in\r\n\r\nchunks".
    let wire = b"4\r\nWiki\r\n5\r\npedia\r\nd\r\n in\r\n\r\nchunks\r\n0\r\n\r\n";
    let body = decode(wire).unwrap();
    assert_eq!(body.payload, b"Wikipedia in\r\n\r\nchunks");
    assert_eq!(body.consumed, wire.len());
}

#[test]
fn payload_crlf_does_not_terminate_its_own_chunk() {
    // Counted, not scanned: a payload full of CRLFs must survive intact.
    let payload = b"\r\n\r\ndata: x\r\n\r\n";
    let wire = encode_body(payload, &[]).unwrap();
    assert_eq!(decode(&wire).unwrap().payload, payload);
}

#[test]
fn bytes_after_the_body_are_left_for_the_caller() {
    let wire = b"1\r\na\r\n0\r\n\r\nGET / HTTP/1.1\r\n";
    let body = decode(wire).unwrap();
    assert_eq!(body.payload, b"a");
    assert_eq!(&wire[body.consumed..], b"GET / HTTP/1.1\r\n");
}

// ------------------------------------------------------- incremental decoding

/// Every proper prefix of a valid chunked body must be `Incomplete`, never `Malformed`.
/// This is the property that makes the codec usable against a socket, where a chunk can
/// arrive split at any byte.
#[test]
fn every_prefix_of_a_valid_body_is_incomplete() {
    let bodies: [&[u8]; 5] = [
        b"0\r\n\r\n",
        b"5\r\nhello\r\n0\r\n\r\n",
        b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n",
        b"3;ext=1\r\nabc\r\n0\r\nX-Sum: 9\r\n\r\n",
        b"2\r\n\r\n\r\n0\r\n\r\n",
    ];
    for body in bodies {
        for split in 0..body.len() {
            match decode(&body[..split]) {
                Err(ChunkedError::Incomplete) => {}
                other => panic!("prefix {split} of {body:?} gave {other:?}"),
            }
        }
        decode(body).expect("the whole body decodes");
    }
}

#[test]
fn incremental_feed_reaches_the_same_answer() {
    let wire = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\nX-Sum: 9\r\n\r\n";
    let mut buffer: Vec<u8> = Vec::new();
    let mut decoded = None;
    for byte in wire {
        buffer.push(*byte);
        if let Ok(body) = decode(&buffer) {
            decoded = Some(body);
        }
    }
    let body = decoded.expect("body completes once every byte has arrived");
    assert_eq!(body.payload, b"Wikipedia");
    assert_eq!(body.trailers, vec![("x-sum".to_string(), "9".to_string())]);
    assert_eq!(body.consumed, wire.len());
}

#[test]
fn incomplete_is_distinct_from_malformed() {
    assert!(matches!(decode(b"5\r\nhel"), Err(ChunkedError::Incomplete)));
    assert_malformed(decode(b"5\r\nhelloXX"), "wrong chunk terminator");
    assert_malformed(decode(b"5\nhello\r\n"), "bare LF after the size");
}

// ------------------------------------------------------------ chunk extensions

#[test]
fn chunk_extensions_are_ignored_not_rejected() {
    let wire = b"5;name=value;flag\r\nhello\r\n0;final=1\r\n\r\n";
    let body = decode(wire).unwrap();
    assert_eq!(body.payload, b"hello");
    assert_eq!(body.consumed, wire.len());
}

#[test]
fn extensions_are_stripped_without_trimming() {
    assert_eq!(strip_extensions(b"1a"), b"1a");
    assert_eq!(strip_extensions(b"1a;q=1"), b"1a");
    assert_eq!(strip_extensions(b";q=1"), b"");
    // Whitespace is preserved so the size parser can reject it explicitly.
    assert_eq!(strip_extensions(b"1a ;q=1"), b"1a ");
    assert_malformed(
        decode(b"5 ;q=1\r\nhello\r\n0\r\n\r\n"),
        "padded size with extension",
    );
}

// -------------------------------------------------------------------- trailers

#[test]
fn trailers_are_parsed_lowercased_and_trimmed() {
    let wire = b"1\r\na\r\n0\r\nX-Sum:  9  \r\nX-Kind: test\r\n\r\n";
    let body = decode(wire).unwrap();
    assert_eq!(
        body.trailers,
        vec![
            ("x-sum".to_string(), "9".to_string()),
            ("x-kind".to_string(), "test".to_string()),
        ]
    );
    assert_eq!(body.consumed, wire.len());
}

#[test]
fn trailers_round_trip_through_the_encoder() {
    let trailers = vec![("X-Checksum".to_string(), "deadbeef".to_string())];
    let wire = encode_body(b"body", &trailers).unwrap();
    assert!(wire.ends_with(b"0\r\nX-Checksum: deadbeef\r\n\r\n"));
    let body = decode(&wire).unwrap();
    assert_eq!(body.payload, b"body");
    assert_eq!(
        body.trailers,
        vec![("x-checksum".to_string(), "deadbeef".to_string())]
    );
}

#[test]
fn malformed_trailer_lines_are_rejected() {
    assert_malformed(decode(b"0\r\nno-colon\r\n\r\n"), "trailer without a colon");
    assert_malformed(decode(b"0\r\n: 9\r\n\r\n"), "trailer with an empty name");
    // Whitespace before the colon is an RFC 9112 §5.1 smuggling divergence, not a typo.
    assert_malformed(decode(b"0\r\nX-Sum : 9\r\n\r\n"), "padded trailer name");
}

#[test]
fn empty_trailer_section_terminates_immediately() {
    assert_eq!(decode_trailers(b"\r\n", 0).unwrap(), (Vec::new(), 2));
}

#[test]
fn too_many_trailers_is_rejected() {
    let mut wire = b"0\r\n".to_vec();
    for index in 0..=MAX_TRAILERS {
        wire.extend_from_slice(format!("X-N{index}: v\r\n").as_bytes());
    }
    wire.extend_from_slice(b"\r\n");
    assert_malformed(decode(&wire), "more trailers than the bound allows");

    let many: Vec<(String, String)> = (0..=MAX_TRAILERS)
        .map(|index| (format!("X-N{index}"), "v".to_string()))
        .collect();
    assert_malformed(
        encode_last_chunk(&many),
        "encoding more trailers than allowed",
    );
}

#[test]
fn exactly_the_trailer_bound_is_accepted() {
    let mut wire = b"0\r\n".to_vec();
    for index in 0..MAX_TRAILERS {
        wire.extend_from_slice(format!("X-N{index}: v\r\n").as_bytes());
    }
    wire.extend_from_slice(b"\r\n");
    assert_eq!(decode(&wire).unwrap().trailers.len(), MAX_TRAILERS);
}

#[test]
fn oversized_trailer_line_is_rejected() {
    let mut wire = b"0\r\nX-Big: ".to_vec();
    wire.extend(std::iter::repeat_n(b'v', 4096));
    wire.extend_from_slice(b"\r\n\r\n");
    assert_malformed(decode(&wire), "trailer line past its bound");
}

#[test]
fn trailer_encoding_refuses_injected_crlf() {
    // Response splitting: a CRLF in a value would forge additional fields.
    let bad = [("X-A".to_string(), "v\r\nX-B: forged".to_string())];
    assert_malformed(
        encode_last_chunk(&bad),
        "CRLF injected into a trailer value",
    );
    let bad_name = [("X A".to_string(), "v".to_string())];
    assert_malformed(encode_last_chunk(&bad_name), "non-token trailer name");
}

// --------------------------------------------------------- strict size grammar

#[test]
fn valid_hex_sizes_parse() {
    assert_eq!(parse_chunk_size(b"0").unwrap(), 0);
    assert_eq!(parse_chunk_size(b"a").unwrap(), 10);
    assert_eq!(parse_chunk_size(b"A").unwrap(), 10);
    assert_eq!(parse_chunk_size(b"1f").unwrap(), 31);
    // Leading zeros are legal and unambiguous, so they are accepted.
    assert_eq!(parse_chunk_size(b"00000005").unwrap(), 5);
    assert_eq!(
        decode(b"0005\r\nhello\r\n0\r\n\r\n").unwrap().payload,
        b"hello"
    );
}

#[test]
fn signed_size_is_rejected() {
    assert_malformed(parse_chunk_size(b"+5"), "plus-signed size");
    assert_malformed(parse_chunk_size(b"-5"), "minus-signed size");
    assert_malformed(
        decode(b"+5\r\nhello\r\n0\r\n\r\n"),
        "plus-signed size on the wire",
    );
    assert_malformed(
        decode(b"-5\r\nhello\r\n0\r\n\r\n"),
        "minus-signed size on the wire",
    );
}

#[test]
fn hex_prefixed_size_is_rejected() {
    assert_malformed(parse_chunk_size(b"0x5"), "0x-prefixed size");
    assert_malformed(parse_chunk_size(b"0X5"), "0X-prefixed size");
    assert_malformed(decode(b"0x5\r\nhello\r\n0\r\n\r\n"), "0x size on the wire");
}

#[test]
fn size_with_whitespace_is_rejected() {
    assert_malformed(parse_chunk_size(b"5 "), "trailing space");
    assert_malformed(parse_chunk_size(b" 5"), "leading space");
    assert_malformed(parse_chunk_size(b"5\t"), "trailing tab");
    assert_malformed(
        decode(b"5 \r\nhello\r\n0\r\n\r\n"),
        "trailing space on the wire",
    );
    assert_malformed(decode(b"0 \r\n\r\n"), "padded zero chunk");
}

#[test]
fn empty_or_non_hex_size_is_rejected() {
    assert_malformed(parse_chunk_size(b""), "empty size");
    assert_malformed(parse_chunk_size(b"5g"), "non-hex digit");
    assert_malformed(decode(b"\r\nhello\r\n"), "missing size");
}

#[test]
fn overflowing_size_is_rejected_not_wrapped() {
    // 17 hex digits cannot fit a 64-bit usize. Had the accumulator wrapped, `1` followed by
    // sixteen zeros would read as 0 and be mistaken for the terminating zero chunk — a
    // smuggled-body primitive. It is refused instead: the running value trips the per-chunk
    // bound on the way up, and `checked_mul`/`checked_add` are the backstop for anything
    // that could otherwise reach `usize::MAX`. Either way the outcome is Malformed, never a
    // silent zero and never a wrapped small value.
    assert_malformed(parse_chunk_size(b"10000000000000000"), "overflowing size");
    assert_malformed(
        decode(b"10000000000000000\r\n\r\n"),
        "overflowing size on the wire",
    );
    assert_malformed(
        parse_chunk_size(b"ffffffffffffffffff"),
        "far-overflowing size",
    );
    assert!(!matches!(parse_chunk_size(b"10000000000000000"), Ok(0)));
}

// ---------------------------------------------------------------------- bounds

#[test]
fn size_above_the_chunk_bound_is_rejected() {
    let over = format!("{:x}", MAX_CHUNK_BYTES + 1);
    assert_malformed(
        parse_chunk_size(over.as_bytes()),
        "size past the chunk bound",
    );
    assert_malformed(
        decode(format!("{over}\r\nhello\r\n").as_bytes()),
        "oversized chunk on the wire",
    );
    // Exactly at the bound is still a legal claim (the body just has not arrived yet).
    let at = format!("{MAX_CHUNK_BYTES:x}");
    assert_eq!(parse_chunk_size(at.as_bytes()).unwrap(), MAX_CHUNK_BYTES);
}

#[test]
fn encoding_an_oversized_single_chunk_is_rejected() {
    let big = vec![0u8; MAX_CHUNK_BYTES + 1];
    assert_malformed(encode_chunk(&big), "chunk payload past the bound");
    // The whole-body helper splits instead of failing, and the result round-trips.
    let wire = encode_body(&big, &[]).expect("encode_body splits oversized payloads");
    assert_eq!(decode(&wire).unwrap().payload.len(), big.len());
}

#[test]
fn oversized_size_line_is_rejected_rather_than_scanned_forever() {
    let mut wire = vec![b'0'; 512];
    wire.extend_from_slice(b"\r\n\r\n");
    assert_malformed(decode(&wire), "size line past its bound");
}

// --------------------------------------------------------------- streaming head

#[test]
fn streaming_head_advertises_chunked_and_never_content_length() {
    let head = streaming_head(200, "OK", "text/event-stream", &[]).unwrap();
    assert!(
        head.starts_with("HTTP/1.1 200 OK\r\n"),
        "status line: {head:?}"
    );
    assert!(head.contains("Transfer-Encoding: chunked\r\n"));
    assert!(head.contains("Content-Type: text/event-stream\r\n"));
    assert!(
        !head.to_ascii_lowercase().contains("content-length"),
        "both framings in one message is the smuggling vector: {head:?}"
    );
    assert!(head.ends_with("\r\n\r\n"), "head ends with a blank line");
}

#[test]
fn caller_supplied_framing_headers_are_dropped() {
    let extra = vec![
        ("Content-Length".to_string(), "0".to_string()),
        ("transfer-encoding".to_string(), "identity".to_string()),
        ("Connection".to_string(), "close".to_string()),
        ("X-Stream".to_string(), "yes".to_string()),
    ];
    let head = streaming_head(200, "OK", "text/plain", &extra).unwrap();
    let lower = head.to_ascii_lowercase();
    assert!(!lower.contains("content-length"));
    assert_eq!(lower.matches("transfer-encoding").count(), 1);
    assert!(!lower.contains("identity"));
    assert_eq!(lower.matches("connection:").count(), 1);
    assert!(
        head.contains("X-Stream: yes\r\n"),
        "non-reserved headers pass through"
    );
}

#[test]
fn streaming_head_refuses_header_injection() {
    let split = vec![("X-A".to_string(), "v\r\nContent-Length: 0".to_string())];
    assert_malformed(
        streaming_head(200, "OK", "text/plain", &split),
        "injected header",
    );
    let bad_name = vec![("X\r\nY".to_string(), "v".to_string())];
    assert_malformed(
        streaming_head(200, "OK", "text/plain", &bad_name),
        "injected name",
    );
    assert_malformed(
        streaming_head(200, "OK", "text/plain\r\nContent-Length: 0", &[]),
        "injected content type",
    );
}

#[test]
fn a_full_streaming_exchange_assembles_correctly() {
    let mut wire = streaming_head(200, "OK", "text/event-stream", &[])
        .unwrap()
        .into_bytes();
    let head_len = wire.len();
    for frame in ["data: one\n\n", "data: two\n\n"] {
        wire.extend_from_slice(&encode_chunk(frame.as_bytes()).unwrap());
    }
    wire.extend_from_slice(&encode_last_chunk(&[]).unwrap());

    let body = decode(&wire[head_len..]).unwrap();
    assert_eq!(body.payload, b"data: one\n\ndata: two\n\n");
    assert_eq!(head_len + body.consumed, wire.len());
}
