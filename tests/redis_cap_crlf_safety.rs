//! A key or value containing CRLF is safe, and this proves why.
//!
//! # The claim
//!
//! A hostile key such as `"a\r\nFLUSHALL\r\n"` cannot become a second command.
//!
//! # Why it holds — length prefixes, not delimiters
//!
//! RESP requests are arrays of *length-prefixed* bulk strings. Each argument goes on the
//! wire as `$<len>\r\n<bytes>\r\n`, and the server reads exactly `len` bytes and then
//! expects the trailing CRLF. It never scans the payload looking for a delimiter, so
//! there is no delimiter for a payload to escape — and therefore nothing to escape
//! *from*. Quoting or stripping CRLF would be the wrong fix for a problem that does not
//! exist, and would corrupt binary values in the process.
//!
//! The vulnerable design is the *inline command* form, where arguments are separated by
//! whitespace and the command by CRLF. `src/redis/encode.rs` deliberately does not offer
//! it, which is what makes this safe by construction rather than by validation.
//!
//! The capability's contribution is to keep the payload as bytes end to end: it never
//! builds a request by concatenating text, so it cannot reintroduce the injection the
//! protocol layer already precludes.
//!
//! # What is asserted
//!
//! Both halves, because either alone is insufficient:
//!
//! 1. The capability's coercion preserves CRLF byte-for-byte rather than stripping,
//!    escaping, or truncating it.
//! 2. Those exact bytes, handed to the real encoder, produce **one** command whose
//!    length prefix accounts for every byte — so the injected text is data.

use std::cell::RefCell;
use std::rc::Rc;

use tetherscript::redis::encode_command;
use tetherscript::redis_cap::coerce_bytes;
use tetherscript::value::Value;

/// A key that would be an injection if requests were built as text.
const HOSTILE_KEY: &str = "a\r\nFLUSHALL\r\n";

/// Coerce a script `str` the way the capability does.
fn coerced(value: &str) -> Vec<u8> {
    coerce_bytes::bytes("redis.get", "key", &Value::Str(Rc::new(value.to_string())))
        .expect("a str key must coerce")
}

/// Coercion preserves CRLF rather than sanitising it.
#[test]
fn coercion_preserves_crlf_byte_for_byte() {
    let key = coerced(HOSTILE_KEY);
    assert_eq!(key, HOSTILE_KEY.as_bytes());
    assert_eq!(key.len(), 13, "no byte was stripped or escaped");
    assert!(key.windows(2).any(|pair| pair == b"\r\n"));
}

/// A CRLF-bearing `bytes` value is likewise untouched.
#[test]
fn coercion_preserves_crlf_in_a_bytes_value() {
    let payload = b"line1\r\nline2\r\n".to_vec();
    let value = Value::Bytes(Rc::new(RefCell::new(payload.clone())));
    assert_eq!(
        coerce_bytes::bytes("redis.set", "value", &value).unwrap(),
        payload
    );
}

/// The encoder turns the hostile key into exactly one argument of one command.
#[test]
fn a_crlf_key_encodes_as_one_length_prefixed_argument() {
    let key = coerced(HOSTILE_KEY);
    let request = encode_command(&[&b"GET"[..], &key]).expect("encode");
    // `*2` — two arguments, not four. The injected `FLUSHALL` is inside argument two.
    assert!(request.starts_with(b"*2\r\n"), "must be a 2-element array");
    assert_eq!(
        request,
        b"*2\r\n$3\r\nGET\r\n$13\r\na\r\nFLUSHALL\r\n\r\n".to_vec(),
        "the $13 prefix accounts for every byte, so the CRLF is data"
    );
}

/// The declared length equals the payload length, which is the whole safety argument.
#[test]
fn the_declared_length_accounts_for_the_crlf_bytes() {
    let key = coerced(HOSTILE_KEY);
    let request = encode_command(&[&b"GET"[..], &key]).expect("encode");
    let declared = format!("${}\r\n", key.len());
    assert!(
        String::from_utf8_lossy(&request).contains(&declared),
        "the header must declare all {} bytes",
        key.len()
    );
}

/// A CRLF-bearing value in a `SET` is likewise one argument.
#[test]
fn a_crlf_value_encodes_as_one_argument() {
    let value = b"a\r\nFLUSHALL".to_vec();
    let request = encode_command(&[&b"SET"[..], &b"k"[..], &value]).expect("encode");
    assert!(request.starts_with(b"*3\r\n"), "three arguments, not five");
    assert_eq!(
        request,
        b"*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$11\r\na\r\nFLUSHALL\r\n".to_vec()
    );
}

/// A NUL byte is equally safe, for the same reason.
#[test]
fn a_nul_byte_in_a_key_is_also_safe() {
    let key = b"a\x00b".to_vec();
    let request = encode_command(&[&b"GET"[..], &key]).expect("encode");
    assert_eq!(request, b"*2\r\n$3\r\nGET\r\n$3\r\na\x00b\r\n".to_vec());
}

/// A key that is *only* CRLF is still one well-formed argument.
#[test]
fn a_key_that_is_only_crlf_is_still_one_argument() {
    let request = encode_command(&[&b"GET"[..], &b"\r\n"[..]]).expect("encode");
    assert_eq!(request, b"*2\r\n$3\r\nGET\r\n$2\r\n\r\n\r\n".to_vec());
}
