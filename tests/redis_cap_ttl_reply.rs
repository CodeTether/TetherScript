//! `TTL`'s three-way answer, and the dispatch surface.
//!
//! Redis encodes `TTL` as a single integer using negative sentinels: `-1` means the key
//! exists but never expires, `-2` means the key does not exist. The capability maps them
//! to distinct *types* rather than passing the sentinels through.
//!
//! # Why the sentinels must not reach a script
//!
//! Handed `-1` and `-2` as plain integers, a script writes the natural thing:
//!
//! ```text
//! if redis.ttl(key)? < 60 { refresh(key) }
//! ```
//!
//! which fires for both sentinels — a key that does not exist, and a key that never
//! expires. Those are precisely the two cases where refreshing is wrong. Distinct types
//! make that comparison a type error instead of a plausible-looking bug.

use tetherscript::redis::Ttl;
use tetherscript::redis_cap::reply_ttl;
use tetherscript::value::Value;

#[test]
fn a_live_ttl_is_an_int_of_seconds() {
    assert!(matches!(reply_ttl::value(Ttl::Seconds(42)), Value::Int(42)));
}

/// A key expiring this very second is still present, so it stays an int.
#[test]
fn a_zero_ttl_is_still_an_int() {
    assert!(matches!(reply_ttl::value(Ttl::Seconds(0)), Value::Int(0)));
}

/// `-1` becomes `false`, not `-1`.
#[test]
fn a_persistent_key_is_false_not_a_negative_int() {
    assert!(matches!(
        reply_ttl::value(Ttl::Persistent),
        Value::Bool(false)
    ));
}

/// `-2` becomes `nil`, matching how a cache miss is spelled everywhere else.
#[test]
fn a_missing_key_is_nil_not_a_negative_int() {
    assert!(matches!(reply_ttl::value(Ttl::Missing), Value::Nil));
}

/// No mapping produces a negative integer, which is the whole point.
#[test]
fn no_outcome_is_ever_a_negative_int() {
    for outcome in [
        Ttl::Seconds(0),
        Ttl::Seconds(5),
        Ttl::Persistent,
        Ttl::Missing,
    ] {
        if let Value::Int(seconds) = reply_ttl::value(outcome) {
            assert!(seconds >= 0, "a sentinel leaked through as {seconds}");
        }
    }
}

/// The three outcomes are mutually distinguishable by type.
#[test]
fn the_three_outcomes_have_three_distinct_types() {
    assert_eq!(reply_ttl::value(Ttl::Seconds(1)).type_name(), "int");
    assert_eq!(reply_ttl::value(Ttl::Persistent).type_name(), "bool");
    assert_eq!(reply_ttl::value(Ttl::Missing).type_name(), "nil");
}

/// The client's own sentinel decoding is the contract this mapping builds on.
#[test]
fn the_client_decodes_the_sentinels_as_documented() {
    assert_eq!(Ttl::from_reply(42), Ttl::Seconds(42));
    assert_eq!(Ttl::from_reply(-1), Ttl::Persistent);
    assert_eq!(Ttl::from_reply(-2), Ttl::Missing);
}

/// End to end: a raw `-2` reply reaches a script as `nil`, never as `-2`.
#[test]
fn a_raw_minus_two_reply_reaches_a_script_as_nil() {
    let mapped = reply_ttl::value(Ttl::from_reply(-2));
    assert!(matches!(mapped, Value::Nil));
}

/// End to end: a raw `-1` reply reaches a script as `false`, never as `-1`.
#[test]
fn a_raw_minus_one_reply_reaches_a_script_as_false() {
    let mapped = reply_ttl::value(Ttl::from_reply(-1));
    assert!(matches!(mapped, Value::Bool(false)));
}
