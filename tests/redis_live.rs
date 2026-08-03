//! Live integration coverage for the native Redis client.
//!
//! These need a real server, because the whole point of the client is wire
//! compatibility: the `AUTH`/`SELECT` handshake, the request/response round trip,
//! and the null-versus-empty distinction cannot be proven against a mock. They are
//! skipped unless `TETHERSCRIPT_REDIS_TEST_URL` is set, so the default `cargo test`
//! run stays hermetic and offline.
//!
//! ```text
//! docker run -d --rm --name ts_redis_test -p 56379:6379 redis:7
//! TETHERSCRIPT_REDIS_TEST_URL=127.0.0.1:56379 cargo test --test redis_live
//! ```
//!
//! Optional environment variables:
//!
//! | Variable | Effect |
//! |---|---|
//! | `TETHERSCRIPT_REDIS_TEST_URL` | `host:port`. Absent means skip everything. |
//! | `TETHERSCRIPT_REDIS_TEST_PASSWORD` | Sent as `AUTH` when set. |
//! | `TETHERSCRIPT_REDIS_TEST_DB` | Database index to `SELECT`; defaults to 0. |
//!
//! Every test namespaces its keys under `tetherscript:test:` and deletes them
//! afterwards, so running against a shared database does not clobber anything and
//! a rerun is not affected by the previous one.

use tetherscript::redis::{Config, Connection, RedisError, RespValue, SetOptions, Ttl};

/// Resolve the test server settings, or `None` when live testing is not enabled.
fn config() -> Option<Config> {
    let target = std::env::var("TETHERSCRIPT_REDIS_TEST_URL").ok()?;
    let mut config = Config::from_address(&target).expect("test URL should parse");
    config.password = std::env::var("TETHERSCRIPT_REDIS_TEST_PASSWORD").ok();
    if let Ok(database) = std::env::var("TETHERSCRIPT_REDIS_TEST_DB") {
        config.database = database.parse().expect("test DB index should be a number");
    }
    Some(config)
}

/// Connect, or return `None` so the test body can skip.
fn connect() -> Option<Connection> {
    let config = config()?;
    Some(Connection::connect(&config).expect("connect should succeed"))
}

/// Namespace a key so a shared server is safe to test against.
fn key(name: &str) -> String {
    format!("tetherscript:test:{name}")
}

/// Remove a key, ignoring whether it existed.
fn cleanup(connection: &mut Connection, name: &str) {
    let owned = key(name);
    connection
        .del(&[owned.as_bytes()])
        .expect("cleanup DEL should succeed");
}

/// The handshake must complete and the server must answer `PING`.
#[test]
fn connects_and_pings() {
    let Some(mut connection) = connect() else {
        return;
    };
    assert_eq!(connection.ping().unwrap(), "PONG");
}

/// A round trip of an arbitrary command through the generic entry point.
#[test]
fn sends_arbitrary_command() {
    let Some(mut connection) = connect() else {
        return;
    };
    let reply = connection
        .command(&[&b"ECHO"[..], &b"round trip"[..]])
        .unwrap();
    assert_eq!(reply, RespValue::Bulk(b"round trip".to_vec()));
}

/// `SET` then `GET` must return exactly what was stored.
#[test]
fn sets_and_gets_a_value() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("set_get");
    connection
        .set(name.as_bytes(), b"value", &SetOptions::default())
        .unwrap();
    assert_eq!(
        connection.get(name.as_bytes()).unwrap(),
        Some(b"value".to_vec())
    );
    cleanup(&mut connection, "set_get");
}

/// A missing key is `None`; a key holding the empty string is `Some(vec![])`. This
/// is the null-bulk-versus-empty-bulk distinction observed end to end.
#[test]
fn distinguishes_missing_key_from_empty_value() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("empty");
    cleanup(&mut connection, "empty");

    assert_eq!(connection.get(name.as_bytes()).unwrap(), None);

    connection
        .set(name.as_bytes(), b"", &SetOptions::default())
        .unwrap();
    assert_eq!(connection.get(name.as_bytes()).unwrap(), Some(Vec::new()));

    cleanup(&mut connection, "empty");
    assert_eq!(connection.get(name.as_bytes()).unwrap(), None);
}

/// A value containing CRLF must survive unchanged. If the client built commands by
/// concatenation this would inject a second command; if it split replies on CRLF
/// this would come back truncated.
#[test]
fn round_trips_a_value_containing_crlf() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("crlf");
    let payload = b"line one\r\nFLUSHALL\r\nline two";
    connection
        .set(name.as_bytes(), payload, &SetOptions::default())
        .unwrap();
    assert_eq!(
        connection.get(name.as_bytes()).unwrap(),
        Some(payload.to_vec())
    );
    // The injected text was data: the database is still populated.
    assert_eq!(connection.exists(&[name.as_bytes()]).unwrap(), 1);
    cleanup(&mut connection, "crlf");
}

/// Binary values, including NUL and invalid UTF-8, must survive.
#[test]
fn round_trips_binary_value() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("binary");
    let payload = vec![0x00, 0xff, 0x0d, 0x0a, 0x80, 0x00];
    connection
        .set(name.as_bytes(), &payload, &SetOptions::default())
        .unwrap();
    assert_eq!(connection.get(name.as_bytes()).unwrap(), Some(payload));
    // The UTF-8 convenience wrapper refuses rather than mangling it.
    assert!(connection.get_str(&name).is_err());
    cleanup(&mut connection, "binary");
}

/// `SET ... NX` returns `true` the first time and `false` afterwards. This is the
/// lock primitive, and it is exactly the null-bulk reply being read correctly.
#[test]
fn set_nx_reports_whether_it_won() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("nx");
    cleanup(&mut connection, "nx");

    let options = SetOptions {
        expire_seconds: Some(60),
        if_not_exists: true,
    };
    assert!(connection.set(name.as_bytes(), b"first", &options).unwrap());
    assert!(!connection
        .set(name.as_bytes(), b"second", &options)
        .unwrap());
    // The loser must not have overwritten the winner.
    assert_eq!(
        connection.get(name.as_bytes()).unwrap(),
        Some(b"first".to_vec())
    );
    cleanup(&mut connection, "nx");
}

/// `SET ... EX` sets value and expiry atomically, so `TTL` is bounded immediately.
#[test]
fn set_ex_applies_an_expiry() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("ex");
    connection
        .set(name.as_bytes(), b"session", &SetOptions::expiring(120))
        .unwrap();
    match connection.ttl(name.as_bytes()).unwrap() {
        Ttl::Seconds(left) => assert!(left > 0 && left <= 120, "unexpected ttl {left}"),
        other => panic!("expected a bounded ttl, got {other:?}"),
    }
    cleanup(&mut connection, "ex");
}

/// `TTL` names all three states, and they must be observably different.
#[test]
fn reports_the_three_ttl_states() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("ttl");
    cleanup(&mut connection, "ttl");

    // Missing.
    assert_eq!(connection.ttl(name.as_bytes()).unwrap(), Ttl::Missing);

    // Present with no expiry.
    connection
        .set(name.as_bytes(), b"v", &SetOptions::default())
        .unwrap();
    assert_eq!(connection.ttl(name.as_bytes()).unwrap(), Ttl::Persistent);

    // Present and bounded.
    assert!(connection.expire(name.as_bytes(), 90).unwrap());
    assert!(matches!(
        connection.ttl(name.as_bytes()).unwrap(),
        Ttl::Seconds(_)
    ));

    cleanup(&mut connection, "ttl");
}

/// `EXPIRE` on an absent key is `false`, not an error.
#[test]
fn expire_on_missing_key_reports_false() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("expire_missing");
    cleanup(&mut connection, "expire_missing");
    assert!(!connection.expire(name.as_bytes(), 60).unwrap());
}

/// `DEL` and `EXISTS` report counts across several keys.
#[test]
fn deletes_and_counts_keys() {
    let Some(mut connection) = connect() else {
        return;
    };
    let first = key("del_a");
    let second = key("del_b");
    let absent = key("del_missing");
    cleanup(&mut connection, "del_missing");

    for name in [&first, &second] {
        connection
            .set(name.as_bytes(), b"v", &SetOptions::default())
            .unwrap();
    }
    assert_eq!(
        connection
            .exists(&[first.as_bytes(), second.as_bytes(), absent.as_bytes()])
            .unwrap(),
        2
    );
    assert_eq!(
        connection
            .del(&[first.as_bytes(), second.as_bytes(), absent.as_bytes()])
            .unwrap(),
        2
    );
    // Idempotent second delete.
    assert_eq!(connection.del(&[first.as_bytes()]).unwrap(), 0);
}

/// A variadic command with no keys is rejected before it reaches the socket, so the
/// connection stays usable afterwards.
#[test]
fn rejects_keyless_del_without_breaking_the_connection() {
    let Some(mut connection) = connect() else {
        return;
    };
    assert!(matches!(
        connection.del(&[]).unwrap_err(),
        RedisError::Protocol(_)
    ));
    assert_eq!(connection.ping().unwrap(), "PONG");
}

/// `INCR` and `INCRBY` are the rate-limit counters: atomic, and creating the key at
/// zero when absent.
#[test]
fn increments_counters() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("counter");
    cleanup(&mut connection, "counter");

    assert_eq!(connection.incr(name.as_bytes()).unwrap(), 1);
    assert_eq!(connection.incr(name.as_bytes()).unwrap(), 2);
    assert_eq!(connection.incrby(name.as_bytes(), 10).unwrap(), 12);
    // A negative delta decrements, which is why no DECRBY helper exists.
    assert_eq!(connection.incrby(name.as_bytes(), -12).unwrap(), 0);

    cleanup(&mut connection, "counter");
}

/// A rate-limit window: increment, and bound the key on the first hit so the
/// counter cannot outlive its window.
#[test]
fn rate_limit_window_survives_as_a_bounded_counter() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("rate");
    cleanup(&mut connection, "rate");

    let hits = connection.incr(name.as_bytes()).unwrap();
    assert_eq!(hits, 1);
    assert!(connection.expire(name.as_bytes(), 60).unwrap());
    assert!(matches!(
        connection.ttl(name.as_bytes()).unwrap(),
        Ttl::Seconds(_)
    ));

    cleanup(&mut connection, "rate");
}

/// An error reply is a protocol-level success: it must surface as a named server
/// error, and the connection must remain usable for the next command.
#[test]
fn surfaces_server_error_and_keeps_the_connection() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("wrongtype");
    cleanup(&mut connection, "wrongtype");

    // Make the key a list, then ask for it as a string.
    connection
        .command(&[&b"RPUSH"[..], name.as_bytes(), &b"element"[..]])
        .unwrap();
    let error = connection.get(name.as_bytes()).unwrap_err();
    assert!(
        error.is_server_error(),
        "expected a server error, got {error}"
    );
    assert_eq!(error.kind(), Some("WRONGTYPE"));
    assert!(error.to_string().contains("redis: server:"));

    // Not a transport failure: the very next command works.
    assert_eq!(connection.ping().unwrap(), "PONG");
    cleanup(&mut connection, "wrongtype");
}

/// An unknown command is also a server error, named by its kind.
#[test]
fn surfaces_unknown_command_as_server_error() {
    let Some(mut connection) = connect() else {
        return;
    };
    let error = connection
        .command(&[&b"TETHERSCRIPT_NOT_A_COMMAND"[..]])
        .unwrap_err();
    assert!(error.is_server_error());
    assert_eq!(error.kind(), Some("ERR"));
    assert_eq!(connection.ping().unwrap(), "PONG");
}

/// `INCR` on a non-numeric string is a server error, not a silent zero.
#[test]
fn surfaces_non_numeric_incr_as_server_error() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("not_a_number");
    connection
        .set(name.as_bytes(), b"abc", &SetOptions::default())
        .unwrap();
    let error = connection.incr(name.as_bytes()).unwrap_err();
    assert!(error.is_server_error());
    cleanup(&mut connection, "not_a_number");
}

/// A value larger than one socket read must reassemble correctly, exercising the
/// incremental read loop against a real server.
#[test]
fn round_trips_a_value_larger_than_one_read() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("large");
    // Deliberately larger than the 8 KiB read chunk, and full of CRLF so a
    // delimiter-based reader would truncate it.
    let payload: Vec<u8> = (0..64 * 1024).map(|index| (index % 251) as u8).collect();
    connection
        .set(name.as_bytes(), &payload, &SetOptions::default())
        .unwrap();
    assert_eq!(connection.get(name.as_bytes()).unwrap(), Some(payload));
    cleanup(&mut connection, "large");
}

/// The string conveniences must agree with the byte APIs.
#[test]
fn string_helpers_match_the_byte_api() {
    let Some(mut connection) = connect() else {
        return;
    };
    let name = key("strings");
    connection
        .set_str(&name, "text value", &SetOptions::default())
        .unwrap();
    assert_eq!(
        connection.get_str(&name).unwrap(),
        Some("text value".to_string())
    );
    cleanup(&mut connection, "strings");
    assert_eq!(connection.get_str(&name).unwrap(), None);
}

/// State must survive across connections — the whole reason the reference
/// application uses Redis rather than an in-process map.
#[test]
fn state_survives_across_connections() {
    let Some(config) = config() else { return };
    let name = key("cross_process");

    {
        let mut writer = Connection::connect(&config).unwrap();
        writer
            .set(name.as_bytes(), b"persisted", &SetOptions::expiring(300))
            .unwrap();
    }

    let mut reader = Connection::connect(&config).unwrap();
    assert_eq!(
        reader.get(name.as_bytes()).unwrap(),
        Some(b"persisted".to_vec())
    );
    reader.del(&[name.as_bytes()]).unwrap();
}

/// A wrong password must be a server error rather than a transport failure, so a
/// caller can tell "misconfigured" from "unreachable".
#[test]
fn rejects_a_wrong_password_as_a_server_error() {
    let Some(mut config) = config() else { return };
    // Only meaningful when the server actually requires a password.
    if config.password.is_none() {
        return;
    }
    config.password = Some("definitely-not-the-password".into());
    let error = Connection::connect(&config).unwrap_err();
    assert!(
        error.is_server_error(),
        "expected an AUTH rejection, got {error}"
    );
}

/// An unreachable port must be a transport error naming the address.
#[test]
fn reports_unreachable_server_as_transport_error() {
    let Some(mut config) = config() else { return };
    // Port 1 is reserved and will not be listening.
    config.port = 1;
    let error = Connection::connect(&config).unwrap_err();
    assert!(!error.is_server_error(), "expected a transport error");
    assert!(error.to_string().contains("redis: transport:"));
}
