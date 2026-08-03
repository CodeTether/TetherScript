//! Behaviour tests for the OAuth 2.0 PKCE and authorization-code built-ins.
//!
//! These drive the built-ins through the interpreter, since that is the surface scripts
//! actually see. Two categories of expected value appear here:
//!
//! * **Fixed vectors.** The PKCE case is the RFC 7636 Appendix B worked example, so the
//!   verifier and its challenge are written down literally. If the S256 derivation ever
//!   drifts, this test fails against the RFC rather than against my own re-derivation.
//! * **Round-trips.** No state token is hardcoded — each is minted by
//!   `oauth_state_token`, and every forged one is built by editing a real token, which is
//!   exactly how an attacker would produce it. A signature cannot be written down in
//!   advance anyway, because each state carries a fresh random nonce.
//!
//! The security-relevant cases assert on the *error text*, not merely on `is_err()`. An
//! open redirect rejected with the wrong message is a rejection the next reader will
//! "fix".

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// The RFC 7636 Appendix B code verifier.
const RFC_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

/// The RFC 7636 Appendix B S256 code challenge for [`RFC_VERIFIER`].
const RFC_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

/// A well-formed config, prepended to every request-building case so each test varies
/// exactly one thing from a known-good baseline.
const CONFIG: &str = r#"
fn base_config() {
    let config = map()
    config.authorize_url = "https://sso.example.com/realms/app/protocol/openid-connect/auth"
    config.client_id = "web-client"
    config.redirect_uri = "https://app.example.com/callback"
    config.scope = "openid profile email"
    config.state = "opaque-state-value"
    config.code_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    return config
}
"#;

/// Run a tetherscript program from source text and return its trimmed stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_oauth_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, source).expect("source should be writable");
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Split output into lines, asserting the expected count so an off-by-one in the script
/// surfaces as a clear failure rather than an index panic.
fn lines(out: &str, expected: usize) -> Vec<&str> {
    let split: Vec<&str> = out.lines().collect();
    assert_eq!(
        split.len(),
        expected,
        "expected {expected} output lines, got:\n{out}"
    );
    split
}

// ---------------------------------------------------------------------------
// PKCE: the RFC 7636 Appendix B vector and the verifier length bounds.
// ---------------------------------------------------------------------------

/// The canonical worked example from RFC 7636 Appendix B. This is the one value in the
/// whole file safe to hardcode, because the RFC fixes it.
#[test]
fn rfc7636_appendix_b_vector() {
    let out = run(&format!(
        r#"
fn main() {{
    println(oauth_pkce_challenge("{RFC_VERIFIER}").unwrap())
}}
"#
    ));
    assert_eq!(out, RFC_CHALLENGE, "RFC 7636 Appendix B S256 challenge");
}

/// 42 characters is one short of the RFC minimum and must be rejected; 43 is the minimum
/// and must be accepted. Testing both sides of the boundary is the only way to catch an
/// off-by-one in the comparison.
#[test]
fn verifier_lower_bound_is_43_not_42() {
    let too_short = &RFC_VERIFIER[..42];
    let out = run(&format!(
        r#"
fn main() {{
    let short = oauth_pkce_challenge("{too_short}")
    println(str(short.is_err()))
    println(short.err())
    println(str(oauth_pkce_challenge("{RFC_VERIFIER}").is_ok()))
}}
"#
    ));
    let out = lines(&out, 3);
    assert_eq!(out[0], "true", "42 characters must be rejected");
    assert!(
        out[1].contains("43-128") && out[1].contains("got 42"),
        "error should name the bounds and the actual length, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "43 characters must be accepted");
}

/// 128 characters is the RFC maximum and must be accepted; 129 must be rejected.
#[test]
fn verifier_upper_bound_is_128_not_129() {
    let at_limit = format!("{RFC_VERIFIER}{RFC_VERIFIER}{}", &RFC_VERIFIER[..42]);
    let over_limit = format!("{RFC_VERIFIER}{RFC_VERIFIER}{RFC_VERIFIER}");
    assert_eq!(at_limit.len(), 128, "fixture should be exactly 128");
    assert_eq!(over_limit.len(), 129, "fixture should be exactly 129");
    let out = run(&format!(
        r#"
fn main() {{
    println(str(oauth_pkce_challenge("{at_limit}").is_ok()))
    let over = oauth_pkce_challenge("{over_limit}")
    println(str(over.is_err()))
    println(over.err())
}}
"#
    ));
    let out = lines(&out, 3);
    assert_eq!(out[0], "true", "128 characters must be accepted");
    assert_eq!(out[1], "true", "129 characters must be rejected");
    assert!(
        out[2].contains("43-128") && out[2].contains("got 129"),
        "error should name the bounds and the actual length, got: {}",
        out[2]
    );
}

/// A generated pair must satisfy the RFC's own rules and be self-consistent: the returned
/// challenge must equal the challenge derived from the returned verifier.
#[test]
fn generated_pair_is_self_consistent_and_43_characters() {
    let out = run(
        r#"
fn main() {
    let pkce = oauth_pkce_pair().unwrap()
    println(str(pkce.code_verifier.len()))
    println(str(pkce.code_challenge.len()))
    println(pkce.code_challenge_method)
    println(str(oauth_pkce_challenge(pkce.code_verifier).unwrap() == pkce.code_challenge))
}
"#,
    );
    assert_eq!(
        out, "43\n43\nS256\ntrue",
        "43-character verifier, 43-character challenge, S256, and consistent"
    );
}

/// Two pairs must differ. A cached or seeded PRNG makes a verifier predictable, and a
/// predictable verifier defeats PKCE entirely.
#[test]
fn successive_pairs_differ() {
    let out = run(
        r#"
fn main() {
    let a = oauth_pkce_pair().unwrap()
    let b = oauth_pkce_pair().unwrap()
    println(str(a.code_verifier == b.code_verifier))
    println(str(a.code_challenge == b.code_challenge))
}
"#,
    );
    assert_eq!(out, "false\nfalse", "two PKCE pairs must not collide");
}

/// A character outside the unreserved set would be percent-encoded in transit, so what the
/// server receives would not be what was hashed.
#[test]
fn verifier_with_reserved_character_is_rejected() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_pkce_challenge("dBjftJeZ4CVP+mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("disallowed character") && out[1].contains("position 12"),
        "error should name the character and its position, got: {}",
        out[1]
    );
}

// ---------------------------------------------------------------------------
// Authorization URL: required parameters, encoding, and secret hygiene.
// ---------------------------------------------------------------------------

/// Every parameter RFC 6749 §4.1.1 and RFC 7636 §4.3 require must be present, and a scope
/// containing spaces must be percent-encoded rather than truncating the query.
#[test]
fn authorize_url_carries_every_required_parameter() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    println(oauth_authorize_url(base_config()).unwrap())
}}
"#
    ));
    for expected in [
        "https://sso.example.com/realms/app/protocol/openid-connect/auth?",
        "response_type=code",
        "client_id=web-client",
        "redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback",
        "scope=openid%20profile%20email",
        "state=opaque-state-value",
        "code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        "code_challenge_method=S256",
    ] {
        assert!(
            out.contains(expected),
            "authorize URL is missing `{expected}`\nfull URL: {out}"
        );
    }
    assert!(
        !out.contains("openid profile"),
        "a raw space would truncate the query at the space: {out}"
    );
}

/// A secret in the authorization URL lands in browser history, `Referer` headers, and
/// access logs. It must not merely be omitted — supplying one must be a hard error, so a
/// misconfiguration is reported rather than silently swallowed.
#[test]
fn client_secret_is_rejected_and_never_appears_in_the_authorize_url() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let config = base_config()
    config.client_secret = "super-secret-value"
    let result = oauth_authorize_url(config)
    println(str(result.is_err()))
    println(result.err())
    let url = oauth_authorize_url(base_config()).unwrap()
    println(str(url.contains("secret")))
    println(str(url.contains("super-secret-value")))
}}
"#
    ));
    let out = lines(&out, 4);
    assert_eq!(
        out[0], "true",
        "a config carrying client_secret must be rejected"
    );
    assert!(
        out[1].contains("client_secret"),
        "error should name client_secret, got: {}",
        out[1]
    );
    assert_eq!(
        out[2], "false",
        "the authorize URL must contain no `secret` parameter at all"
    );
    assert_eq!(out[3], "false", "no secret value may appear in the URL");
}

/// A state-less or PKCE-less authorization request is exactly the vulnerable flow this
/// group exists to prevent, so omitting either field must be an error, not a silent
/// downgrade.
#[test]
fn state_and_code_challenge_are_mandatory() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let no_state = base_config()
    no_state.state = nil
    let a = oauth_authorize_url(no_state)
    println(str(a.is_err()))
    println(a.err())
    let no_pkce = base_config()
    no_pkce.code_challenge = nil
    let b = oauth_authorize_url(no_pkce)
    println(str(b.is_err()))
    println(b.err())
}}
"#
    ));
    let out = lines(&out, 4);
    assert_eq!(out[0], "true", "missing state must be an error");
    assert!(
        out[1].contains("state"),
        "error should name state, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "missing code_challenge must be an error");
    assert!(
        out[3].contains("code_challenge"),
        "error should name code_challenge, got: {}",
        out[3]
    );
}

/// An empty string is what a script naturally produces for a value it did not have.
/// Accepting it would emit `state=`, which is a state-less request wearing a state
/// parameter.
#[test]
fn empty_state_is_rejected_as_firmly_as_a_missing_one() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let config = base_config()
    config.state = ""
    let result = oauth_authorize_url(config)
    println(str(result.is_err()))
    println(result.err())
}}
"#
    ));
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("state") && out[1].contains("empty"),
        "error should say the state must not be empty, got: {}",
        out[1]
    );
}

/// A `redirect_uri` with a fragment cannot be compared exactly, because the server never
/// receives the fragment. Rejecting it locally names the real problem.
#[test]
fn redirect_uri_with_a_fragment_is_rejected() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let config = base_config()
    config.redirect_uri = "https://app.example.com/callback#frag"
    let result = oauth_authorize_url(config)
    println(str(result.is_err()))
    println(result.err())
}}
"#
    ));
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("fragment"),
        "error should name the fragment, got: {}",
        out[1]
    );
}

/// Plaintext `http` on a routable host exposes the authorization code to the network;
/// loopback is the standard native-app exception and must still work.
#[test]
fn plaintext_redirect_uri_is_loopback_only() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let routable = base_config()
    routable.redirect_uri = "http://app.example.com/callback"
    let bad = oauth_authorize_url(routable)
    println(str(bad.is_err()))
    println(bad.err())
    let loopback = base_config()
    loopback.redirect_uri = "http://127.0.0.1:8080/callback"
    println(str(oauth_authorize_url(loopback).is_ok()))
}}
"#
    ));
    let out = lines(&out, 3);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("loopback"),
        "error should explain the loopback exception, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "loopback http must be accepted");
}

/// A discovery document may hand back an endpoint that already carries a query, so the
/// join must use `&` rather than emitting a second `?`.
#[test]
fn existing_query_on_the_authorize_endpoint_is_joined_with_an_ampersand() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let config = base_config()
    config.authorize_url = "https://sso.example.com/auth?realm=app"
    println(oauth_authorize_url(config).unwrap())
}}
"#
    ));
    assert!(
        out.contains("?realm=app&response_type=code"),
        "an existing query must be extended with `&`, got: {out}"
    );
}

// ---------------------------------------------------------------------------
// State: round-trip, expiry, and tampering.
// ---------------------------------------------------------------------------

/// The basic contract: a state minted here verifies here, and hands back the exact path it
/// was given.
#[test]
fn state_round_trips_and_returns_the_original_path() {
    let out = run(
        r#"
fn main() {
    let token = oauth_state_token("shared-secret", 600, "/dashboard/reports").unwrap()
    println(oauth_state_verify("shared-secret", token).unwrap())
    println(str(token.contains("=")))
    println(str(token.contains("+")))
    println(str(token.contains("/")))
}
"#,
    );
    assert_eq!(
        out, "/dashboard/reports\nfalse\nfalse\nfalse",
        "the path must survive, and the token must be URL-safe unescaped"
    );
}

/// A path containing `.` must survive, because `.` is the payload field separator. This is
/// why `return_to` is base64url-encoded inside the payload.
#[test]
fn a_dot_in_the_return_path_does_not_shift_the_payload_fields() {
    let out = run(
        r#"
fn main() {
    println(oauth_state_verify("s", oauth_state_token("s", 600, "/files/report.v2.pdf").unwrap()).unwrap())
}
"#,
    );
    assert_eq!(out, "/files/report.v2.pdf");
}

/// A one-second TTL is used and waited out. That is slow but honest: forging a past `exp`
/// would require signing a payload by hand, which tests my re-implementation rather than
/// the built-in.
#[test]
fn expired_state_is_rejected_with_a_named_error() {
    let out = run(
        r#"
fn main() {
    let token = oauth_state_token("shared-secret", 1, "/dashboard").unwrap()
    println(str(oauth_state_verify("shared-secret", token).is_ok()))
    sleep_ms(1600)
    let after = oauth_state_verify("shared-secret", token)
    println(str(after.is_err()))
    println(after.err())
}
"#,
    );
    let out = lines(&out, 3);
    assert_eq!(out[0], "true", "a fresh state must verify");
    assert_eq!(out[1], "true", "an expired state must be an error");
    assert!(
        out[2].contains("expired state"),
        "error should say the state expired, not that it was forged, got: {}",
        out[2]
    );
}

/// Editing the payload invalidates the signature that covers it, and the failure must be
/// distinguishable from an expiry: one is an attack, the other is a slow user.
#[test]
fn tampered_state_fails_the_signature_check() {
    let out = run(
        r#"
fn main() {
    let token = oauth_state_token("shared-secret", 600, "/dashboard").unwrap()
    let parts = token.split(".")
    // A different but well-formed base64url payload, keeping the real signature.
    let forged = "djEuZGVhZGJlZWYuMS4yLkx3" + "." + parts[1]
    let checked = oauth_state_verify("shared-secret", forged)
    println(str(checked.is_err()))
    println(checked.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("bad signature"),
        "a re-encoded payload must fail the signature, got: {}",
        out[1]
    );
}

/// The wrong secret must be a bad-signature error, not a quiet `false`: it means either
/// forgery or a misconfigured deployment, and both need naming.
#[test]
fn wrong_secret_is_a_bad_signature_error() {
    let out = run(
        r#"
fn main() {
    let token = oauth_state_token("shared-secret", 600, "/dashboard").unwrap()
    let checked = oauth_state_verify("other-secret", token)
    println(str(checked.is_err()))
    println(checked.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("bad signature"),
        "error should name the bad signature, got: {}",
        out[1]
    );
}

/// A malformed value did not come from here at all, and must say so rather than being
/// reported as forgery.
#[test]
fn malformed_state_names_the_segment_count() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_state_verify("shared-secret", "only-one-segment")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("malformed state"),
        "error should say malformed state, got: {}",
        out[1]
    );
}

/// A fresh nonce per state is what stops one token being guessed from another, and two
/// states minted in the same second must still differ.
#[test]
fn successive_states_differ_because_of_the_nonce() {
    let out = run(
        r#"
fn main() {
    let a = oauth_state_token("s", 600, "/x").unwrap()
    let b = oauth_state_token("s", 600, "/x").unwrap()
    println(str(a == b))
}
"#,
    );
    assert_eq!(out, "false", "two states must not collide");
}

/// A non-positive TTL would mint a state that is already dead, or one that never expires
/// depending on the comparison. Neither is a sane request, so it is rejected.
#[test]
fn non_positive_ttl_is_rejected() {
    let out = run(
        r#"
fn main() {
    let zero = oauth_state_token("s", 0, "/x")
    println(str(zero.is_err()))
    println(zero.err())
    println(str(oauth_state_token("s", -5, "/x").is_err()))
}
"#,
    );
    let out = lines(&out, 3);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("bad ttl_secs"),
        "error should name the bad ttl, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "a negative ttl must also be rejected");
}

// ---------------------------------------------------------------------------
// return_to: the open-redirect class. Each disguise gets its own case.
// ---------------------------------------------------------------------------

/// The straightforward absolute URL. This is the form a naive `startsWith("http")` check
/// catches, and the only one it catches.
#[test]
fn absolute_url_return_to_is_rejected() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_state_token("s", 600, "http://evil.example/harvest")
    println(str(bad.is_err()))
    println(bad.err())
    println(str(oauth_state_token("s", 600, "https://evil.example/harvest").is_err()))
}
"#,
    );
    let out = lines(&out, 3);
    assert_eq!(out[0], "true", "http://evil must be rejected");
    assert!(
        out[1].contains("relative path") && out[1].contains("open redirect"),
        "error should name the open-redirect class, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "https://evil must be rejected too");
}

/// The scheme-relative form. This one *starts with a slash*, so it slips past a "must
/// begin with /" check while still navigating cross-origin.
#[test]
fn scheme_relative_return_to_is_rejected() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_state_token("s", 600, "//evil.example/harvest")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true", "//evil must be rejected");
    assert!(
        out[1].contains("scheme-relative"),
        "error should name the scheme-relative form, got: {}",
        out[1]
    );
}

/// The backslash disguise. Browsers normalise `\` to `/` in the authority position, so
/// each of these is a scheme-relative URL that passes both checks above.
#[test]
fn backslash_return_to_forms_are_rejected() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_state_token("s", 600, "/\\evil.example/harvest")
    println(str(bad.is_err()))
    println(bad.err())
    println(str(oauth_state_token("s", 600, "\\\\evil.example/harvest").is_err()))
    println(str(oauth_state_token("s", 600, "/\\/evil.example").is_err()))
}
"#,
    );
    let out = lines(&out, 4);
    assert_eq!(out[0], "true", "/\\evil must be rejected");
    assert!(
        out[1].contains("backslash"),
        "error should name the backslash, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "\\\\evil must be rejected");
    assert_eq!(out[3], "true", "/\\/evil must be rejected");
}

/// The counterpart to the rejections: an ordinary relative path must work, or the defence
/// is just a broken feature.
#[test]
fn relative_return_to_is_accepted() {
    let out = run(
        r#"
fn main() {
    println(str(oauth_state_token("s", 600, "/").is_ok()))
    println(str(oauth_state_token("s", 600, "/dashboard").is_ok()))
    println(str(oauth_state_token("s", 600, "/a/b/c?tab=2").is_ok()))
    println(oauth_state_verify("s", oauth_state_token("s", 600, "/a/b/c?tab=2").unwrap()).unwrap())
}
"#,
    );
    assert_eq!(out, "true\ntrue\ntrue\n/a/b/c?tab=2");
}

/// A `return_to` carrying `\r` or `\n` becomes a header-injection vector the moment it is
/// written into a `Location` response header.
#[test]
fn return_to_with_a_control_character_is_rejected() {
    let out = run(
        r#"
fn main() {
    let bad = oauth_state_token("s", 600, "/ok\r\nLocation: https://evil.example")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("inject a response header"),
        "error should name the header-injection risk, got: {}",
        out[1]
    );
}

// ---------------------------------------------------------------------------
// Callback: an error must never be mistaken for a success.
// ---------------------------------------------------------------------------

/// The headline case. A handler that reads only `code` sees `nil` and reports the wrong
/// thing much later; here the error callback is an `Err` carrying what the provider said.
#[test]
fn error_callback_is_surfaced_as_an_error_not_a_missing_code() {
    let out = run(
        r#"
fn main() {
    let result = oauth_callback_params("error=access_denied&error_description=User+declined&state=abc")
    println(str(result.is_err()))
    println(result.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(
        out[0], "true",
        "an error callback must not parse as a success"
    );
    assert!(
        out[1].contains("access_denied"),
        "error should carry the provider's error code, got: {}",
        out[1]
    );
    assert!(
        out[1].contains("User declined"),
        "error should carry the provider's description, decoded, got: {}",
        out[1]
    );
}

/// An `error` with no description must still be an error, and must still name the code.
#[test]
fn error_callback_without_a_description_still_errors() {
    let out = run(
        r#"
fn main() {
    let result = oauth_callback_params("?error=consent_required")
    println(str(result.is_err()))
    println(result.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("consent_required"),
        "error should name the code, got: {}",
        out[1]
    );
}

/// A success callback yields all four fields, with the absent ones `nil` rather than
/// missing keys, so a script can read any of them without a containment check.
#[test]
fn success_callback_yields_all_four_fields() {
    let out = run(
        r#"
fn main() {
    let params = oauth_callback_params("code=auth-code-123&state=state-value").unwrap()
    println(params.code)
    println(params.state)
    println(str(params.error == nil))
    println(str(params.error_description == nil))
}
"#,
    );
    assert_eq!(out, "auth-code-123\nstate-value\ntrue\ntrue");
}

/// An empty callback is not a successful login. Reporting it as "neither code nor error"
/// names the actual problem instead of a downstream `invalid_grant`.
#[test]
fn callback_with_neither_code_nor_error_is_an_error() {
    let out = run(
        r#"
fn main() {
    let result = oauth_callback_params("session_state=xyz")
    println(str(result.is_err()))
    println(result.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("neither") && out[1].contains("code"),
        "error should say neither code nor error was present, got: {}",
        out[1]
    );
}

/// A conforming server sends `code` or `error`, never both. A callback carrying both is a
/// tampered redirect, and the failure signal is the one that must not be lost.
#[test]
fn callback_with_both_code_and_error_is_treated_as_an_error() {
    let out = run(
        r#"
fn main() {
    let result = oauth_callback_params("code=abc&error=access_denied")
    println(str(result.is_err()))
}
"#,
    );
    assert_eq!(out, "true", "the error must win over the code");
}

/// A crafted `?code=good&code=evil` must not let the second value override the first.
#[test]
fn only_the_first_occurrence_of_a_field_is_used() {
    let out = run(
        r#"
fn main() {
    println(oauth_callback_params("code=first&code=second").unwrap().code)
}
"#,
    );
    assert_eq!(out, "first", "a duplicated parameter must not override");
}

/// Values are percent-decoded, and a leading `?` is tolerated so a script may pass a URL's
/// tail by hand without the first parameter name becoming `?code`.
#[test]
fn callback_values_are_percent_decoded_and_a_leading_question_mark_is_tolerated() {
    let out = run(
        r#"
fn main() {
    let params = oauth_callback_params("?code=a%2Fb%20c&state=x%3Dy").unwrap()
    println(params.code)
    println(params.state)
}
"#,
    );
    assert_eq!(out, "a/b c\nx=y");
}

/// A malformed escape must be an error, not a silent pass-through: a value a script sees
/// that differs from the value the provider sent is how a decoding bug becomes a bypass.
#[test]
fn malformed_percent_escape_in_a_callback_value_is_an_error() {
    let out = run(
        r#"
fn main() {
    let result = oauth_callback_params("code=abc%zz")
    println(str(result.is_err()))
    println(result.err())
}
"#,
    );
    let out = lines(&out, 2);
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("percent escape"),
        "error should name the bad escape, got: {}",
        out[1]
    );
}

// ---------------------------------------------------------------------------
// Token request body: where the secret belongs, and the redirect_uri echo.
// ---------------------------------------------------------------------------

/// The token body must carry every parameter RFC 6749 §4.1.3 and RFC 7636 §4.5 require,
/// and must echo the *same* `redirect_uri` the authorization request used.
#[test]
fn token_request_body_carries_the_grant_and_the_verifier() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let body = oauth_token_request_body(base_config(), "auth-code-123", "{RFC_VERIFIER}").unwrap()
    println(body)
}}
"#
    ));
    for expected in [
        "grant_type=authorization_code",
        "code=auth-code-123",
        "redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback",
        "client_id=web-client",
        &format!("code_verifier={RFC_VERIFIER}"),
    ] {
        assert!(
            out.contains(expected),
            "token body is missing `{expected}`\nfull body: {out}"
        );
    }
}

/// A public client has no secret and must still be able to exchange the code; PKCE is what
/// protects it. A confidential client's secret must appear here — and only here.
#[test]
fn client_secret_appears_in_the_token_body_but_is_optional() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let public = oauth_token_request_body(base_config(), "c", "{RFC_VERIFIER}").unwrap()
    println(str(public.contains("client_secret")))
    let config = base_config()
    config.client_secret = "s3cr3t"
    let confidential = oauth_token_request_body(config, "c", "{RFC_VERIFIER}").unwrap()
    println(str(confidential.contains("client_secret=s3cr3t")))
}}
"#
    ));
    assert_eq!(
        out, "false\ntrue",
        "the secret must be optional in the body, and present when configured"
    );
}

/// A verifier outside the RFC range must fail locally with a clear message, rather than as
/// an opaque `invalid_grant` from the provider several network hops later.
#[test]
fn token_request_body_validates_the_verifier_length() {
    let too_short = &RFC_VERIFIER[..42];
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let bad = oauth_token_request_body(base_config(), "code", "{too_short}")
    println(str(bad.is_err()))
    println(bad.err())
    let empty = oauth_token_request_body(base_config(), "", "{RFC_VERIFIER}")
    println(str(empty.is_err()))
    println(empty.err())
}}
"#
    ));
    let out = lines(&out, 4);
    assert_eq!(out[0], "true", "a 42-character verifier must be rejected");
    assert!(
        out[1].contains("43-128") && out[1].contains("got 42"),
        "error should name the bounds, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "an empty code must be rejected");
    assert!(
        out[3].contains("code must not be empty"),
        "error should name the empty code, got: {}",
        out[3]
    );
}

/// The `redirect_uri` must be byte-identical in both requests. Reading it from one config
/// field is what guarantees that, so this asserts the two encodings match exactly.
#[test]
fn redirect_uri_is_identical_in_the_authorize_url_and_the_token_body() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let config = base_config()
    let url = oauth_authorize_url(config).unwrap()
    let body = oauth_token_request_body(config, "c", "{RFC_VERIFIER}").unwrap()
    let encoded = "redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback"
    println(str(url.contains(encoded)))
    println(str(body.contains(encoded)))
}}
"#
    ));
    assert_eq!(
        out, "true\ntrue",
        "both requests must carry the identical redirect_uri encoding"
    );
}

// ---------------------------------------------------------------------------
// End to end: the shape a real handler pair would use.
// ---------------------------------------------------------------------------

/// Mint, redirect, come back, verify, exchange. This is the flow the reference application
/// runs against Keycloak, minus the network.
#[test]
fn full_flow_from_pkce_pair_to_token_body() {
    let out = run(&format!(
        r#"{CONFIG}
fn main() {{
    let secret = "deployment-signing-key"
    let pkce = oauth_pkce_pair().unwrap()
    let config = base_config()
    config.state = oauth_state_token(secret, 600, "/dashboard").unwrap()
    config.code_challenge = pkce.code_challenge
    let url = oauth_authorize_url(config).unwrap()
    println(str(url.contains("code_challenge_method=S256")))

    // The provider redirects back with the state it was given.
    let query = "code=returned-code&state=" + config.state
    let params = oauth_callback_params(query).unwrap()
    println(oauth_state_verify(secret, params.state).unwrap())

    let body = oauth_token_request_body(config, params.code, pkce.code_verifier).unwrap()
    println(str(body.contains("code=returned-code")))
    println(str(body.contains("code_verifier=" + pkce.code_verifier)))
}}
"#
    ));
    assert_eq!(
        out, "true\n/dashboard\ntrue\ntrue",
        "the whole flow must round-trip"
    );
}
