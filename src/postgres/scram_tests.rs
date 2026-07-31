//! SCRAM-SHA-256 coverage against the published RFC 7677 exchange.
//!
//! RFC 7677 section 3 gives a complete client/server transcript for user "user"
//! with password "pencil". Reproducing its client-final proof exercises PBKDF2,
//! HMAC, the auth-message concatenation, and base64 together.

use super::scram::{client_final, parse_server_first};

const CLIENT_NONCE: &str = "rOprNGfwEbeRWgbNEkqO";
const SERVER_FIRST: &str =
    "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";

#[test]
fn parses_server_first_attributes() {
    let first = parse_server_first(SERVER_FIRST).expect("server-first should parse");
    assert_eq!(first.iterations, 4096);
    assert_eq!(first.salt.len(), 16);
    assert!(first.nonce.starts_with(CLIENT_NONCE));
}

/// The proof in RFC 7677 is `dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ=`.
#[test]
fn client_final_matches_rfc7677_proof() {
    let first = parse_server_first(SERVER_FIRST).expect("server-first should parse");
    let final_message = client_final(
        "pencil",
        CLIENT_NONCE,
        &first,
        &format!("n=user,r={CLIENT_NONCE}"),
        SERVER_FIRST,
    )
    .expect("client-final should build");
    assert_eq!(
        final_message,
        "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
         p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
    );
}

/// A server that echoes a nonce not extending ours is a MITM signal.
#[test]
fn rejects_server_nonce_that_does_not_extend_client_nonce() {
    let first = parse_server_first("r=someoneelse,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
        .expect("server-first should parse");
    let error = client_final("pencil", CLIENT_NONCE, &first, "n=user,r=x", "r=x")
        .expect_err("mismatched nonce must be rejected");
    assert!(error.contains("does not extend"), "got: {error}");
}

#[test]
fn missing_iteration_count_is_named_in_the_error() {
    let error =
        parse_server_first("r=abc,s=W22ZaJ0SNY7soEsUEjb6gQ==").expect_err("missing i= must fail");
    assert!(error.contains("iteration count"), "got: {error}");
}
