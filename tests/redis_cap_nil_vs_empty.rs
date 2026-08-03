//! A cache miss is not a cached empty value.
//!
//! This is the distinction the capability exists to preserve, so it gets its own file.
//!
//! # What breaks when they are flattened
//!
//! Redis answers a `GET` for a missing key with `$-1\r\n` and a `GET` for a key holding
//! the empty string with `$0\r\n\r\n`. The client models that as
//! `Option<Vec<u8>>`, and the capability must carry it through to the script:
//!
//! * **Session store.** Flattened to `nil`, a user whose session value is legitimately
//!   empty is indistinguishable from a logged-out one, so they are logged out.
//! * **Render cache.** Flattened to `""`, a page that legitimately renders to nothing
//!   is never treated as cached, so it re-renders on every request forever.
//!
//! Both are silent. Neither shows up as an error. Hence these tests.

use tetherscript::redis_cap::reply;
use tetherscript::value::Value;

/// A missing key is `nil`.
#[test]
fn a_missing_key_maps_to_nil() {
    assert!(matches!(reply::optional_bulk(None), Value::Nil));
}

/// A key holding the empty string is an empty `str`, not `nil`.
#[test]
fn an_empty_value_maps_to_an_empty_str_not_nil() {
    match reply::optional_bulk(Some(Vec::new())) {
        Value::Str(text) => assert_eq!(text.as_str(), ""),
        other => panic!("expected an empty str, got {}", other.type_name()),
    }
}

/// The two must be distinguishable by type, which is how a script tests them.
#[test]
fn a_miss_and_an_empty_hit_are_different_types() {
    let miss = reply::optional_bulk(None);
    let empty_hit = reply::optional_bulk(Some(Vec::new()));
    assert_eq!(miss.type_name(), "nil");
    assert_eq!(empty_hit.type_name(), "str");
    assert_ne!(
        miss.type_name(),
        empty_hit.type_name(),
        "a cache miss and a cached empty value must not be the same value"
    );
}

/// `truthy` is *not* the way to test for a miss: both are falsey.
///
/// Asserted so the trap is documented by a test rather than discovered in production.
#[test]
fn truthiness_cannot_distinguish_them_so_scripts_must_compare_to_nil() {
    assert!(!reply::optional_bulk(None).truthy());
    assert!(!reply::optional_bulk(Some(Vec::new())).truthy());
}

/// A present, non-empty value round-trips as a `str`.
#[test]
fn a_present_value_maps_to_a_str() {
    match reply::optional_bulk(Some(b"hello".to_vec())) {
        Value::Str(text) => assert_eq!(text.as_str(), "hello"),
        other => panic!("expected a str, got {}", other.type_name()),
    }
}

/// A non-UTF-8 value becomes `bytes` rather than being lossily decoded or erroring.
///
/// A render cache storing a PNG must get its bytes back unchanged.
#[test]
fn a_binary_value_maps_to_bytes_without_loss() {
    let png_magic = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    match reply::optional_bulk(Some(png_magic.clone())) {
        Value::Bytes(bytes) => assert_eq!(*bytes.borrow(), png_magic),
        other => panic!("expected bytes, got {}", other.type_name()),
    }
}

/// A value that is only *partly* invalid still keeps every byte.
#[test]
fn a_partly_invalid_value_keeps_all_its_bytes() {
    let mixed = vec![b'o', b'k', 0xff];
    match reply::optional_bulk(Some(mixed.clone())) {
        Value::Bytes(bytes) => assert_eq!(*bytes.borrow(), mixed),
        other => panic!("expected bytes, got {}", other.type_name()),
    }
}
