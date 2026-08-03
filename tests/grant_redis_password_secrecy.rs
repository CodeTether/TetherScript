//! The `--grant-redis` URL parser must never disclose the password.
//!
//! Split from `grant_redis_url.rs` so this property is legible on its own: it is not a
//! parsing rule, it is the secrecy invariant of the grant.
//!
//! # Why this needs a test rather than a code review
//!
//! A `--grant-redis` URL carries a credential, and its rejections reach stderr, shell
//! history, log aggregators, and CI output. The tempting implementation —
//! `format!("bad URL: {url}")` — publishes the password of every malformed grant, and
//! it is exactly what `main_caps::db`'s missing-scheme message does today
//! (`--grant-db must start with postgres:// (got `{url}`)`). That is a latent leak the
//! Redis parser deliberately does not copy, and a test is the only thing that stops it
//! being reintroduced by someone improving the diagnostics.
//!
//! The password used here is distinctive so a match cannot be coincidence.

use tetherscript::redis_cap::url;

/// A password no error message could contain by accident.
const SECRET: &str = "hunter2-Zq9x-correcthorse";

/// Extract the rejection message.
///
/// [`Config`](tetherscript::redis::Config) does not derive `Debug` — precisely because
/// it holds a password — so `expect_err` is unavailable.
fn rejection(target: &str) -> String {
    match url::parse(target) {
        Ok(_) => panic!("this URL must be rejected"),
        Err(error) => error,
    }
}

/// Every rejection path, with a password attached to each.
fn malformed_urls() -> Vec<String> {
    vec![
        // Scheme failures: the URL is present and must not be quoted back.
        format!("rediss://app:{SECRET}@cache.internal:6380/0"),
        format!("ftp://app:{SECRET}@cache/0"),
        format!("app:{SECRET}@cache/0"),
        // Authority failures.
        format!("redis://app:{SECRET}@cache:not-a-port/0"),
        format!("redis://app:{SECRET}@cache:70000/0"),
        format!("redis://app:{SECRET}@/0"),
        format!("redis://app:{SECRET}@:6379/0"),
        // Database failures.
        format!("redis://app:{SECRET}@cache/sessions"),
        format!("redis://app:{SECRET}@cache/-1"),
        format!("redis://app:{SECRET}@cache/0/extra"),
    ]
}

#[test]
fn no_rejection_echoes_the_password() {
    for target in malformed_urls() {
        let error = rejection(&target);
        assert!(
            !error.contains(SECRET),
            "a rejection leaked the password: {error}"
        );
    }
}

/// Quoting the whole URL is how the password leaks by accident, so it is refused too.
#[test]
fn no_rejection_echoes_the_whole_url() {
    for target in malformed_urls() {
        let error = rejection(&target);
        assert!(
            !error.contains(&target),
            "a rejection quoted the whole URL: {error}"
        );
    }
}

/// Nor may a rejection quote the credential component around the password.
#[test]
fn no_rejection_echoes_the_credentials() {
    for target in malformed_urls() {
        let error = rejection(&target);
        assert!(
            !error.contains("app:"),
            "a rejection quoted the credentials: {error}"
        );
    }
}

/// The `rediss://` refusal is the most likely place to leak, so it is asserted alone:
/// it is the one rejection whose *purpose* is protecting that password.
#[test]
fn the_rediss_refusal_explains_itself_without_the_password() {
    let error = rejection(&format!("rediss://app:{SECRET}@cache:6380/0"));
    assert!(!error.contains(SECRET), "leaked the password: {error}");
    assert!(error.contains("TLS"), "should name TLS: {error}");
    assert!(error.contains("cleartext"), "should say why: {error}");
}
