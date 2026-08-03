//! A non-positive TTL is refused, naming the parameter.
//!
//! # Why this is a security property and not a nicety
//!
//! Redis reads `EXPIRE key 0` and `EXPIRE key -1` as **delete the key now**. So a
//! capability that forwarded whatever number it was handed turns an arithmetic slip in
//! a script into silent data loss:
//!
//! * A rate limiter computing `window_end - now` gets `0` at the boundary, or a negative
//!   number under clock skew, and *clears its own counter* instead of throttling — the
//!   exact moment throttling matters.
//! * A session store refreshing a TTL from a stale timestamp logs the user out.
//!
//! Neither raises an error at the server; both look like success. `SETEX key 0 v` is at
//! least refused by the server, but only after a round trip and with a message that does
//! not name the script's parameter.
//!
//! So the capability refuses locally, before any byte reaches the socket, and says which
//! parameter was wrong. Deletion remains available deliberately, through `redis.del`.

use tetherscript::redis_cap::coerce_seconds;

#[test]
fn a_positive_ttl_is_accepted() {
    assert_eq!(
        coerce_seconds::positive("redis.expire", "seconds", 3600).unwrap(),
        3600
    );
}

#[test]
fn one_second_is_accepted_as_the_smallest_usable_ttl() {
    assert_eq!(
        coerce_seconds::positive("redis.setex", "seconds", 1).unwrap(),
        1
    );
}

/// Zero would delete the key.
#[test]
fn a_zero_ttl_is_rejected() {
    let error = coerce_seconds::positive("redis.expire", "seconds", 0).unwrap_err();
    assert!(
        error.contains("`seconds`"),
        "must name the parameter: {error}"
    );
    assert!(error.contains("positive"), "got: {error}");
}

/// A negative TTL would also delete the key.
#[test]
fn a_negative_ttl_is_rejected() {
    let error = coerce_seconds::positive("redis.setex", "seconds", -1).unwrap_err();
    assert!(
        error.contains("`seconds`"),
        "must name the parameter: {error}"
    );
}

#[test]
fn a_large_negative_ttl_is_rejected() {
    assert!(coerce_seconds::positive("redis.expire", "seconds", i64::MIN).is_err());
}

/// The rejection names the method too, so a caller knows which call to fix.
#[test]
fn the_rejection_names_the_method() {
    let error = coerce_seconds::positive("redis.expire", "seconds", 0).unwrap_err();
    assert!(error.starts_with("redis.expire:"), "got: {error}");
}

/// The rejection points at the operation the caller may actually have meant.
#[test]
fn the_rejection_suggests_del_for_deletion() {
    let error = coerce_seconds::positive("redis.expire", "seconds", 0).unwrap_err();
    assert!(
        error.contains("redis.del"),
        "if the caller meant to delete, say so: {error}"
    );
}

/// The offending number is echoed; a duration is not a secret, unlike a value.
#[test]
fn the_rejection_echoes_the_offending_number() {
    let error = coerce_seconds::positive("redis.setex", "seconds", -42).unwrap_err();
    assert!(error.contains("-42"), "got: {error}");
}
