//! Behaviour tests for RS256 JWT claim validation.
//!
//! # What is under test, and what is not
//!
//! Claim validation only. Signature *authenticity* is decided by a
//! [`StubVerifier`], which accepts one literal byte string, so there is no RSA key
//! material and no cryptographic assertion here. That split is the design: the
//! module under test performs no RSA arithmetic, no JWKS parsing, and no HTTP.
//!
//! What the stub *does* let these tests assert precisely is the property that
//! matters most — that a refused signature yields no claims at all, and that the
//! claim checks are unreachable until the signature is accepted.
//!
//! # Fixtures
//!
//! Tokens are built with [`token_with`], which base64url-encodes literal header and
//! payload JSON verbatim. Encoding the JSON *as written* is deliberate: it lets a
//! test present a non-object payload, a header with no `alg`, or standard-base64
//! bytes, none of which a well-behaved builder API would produce.
//!
//! Where a test needs standard-base64 or an extra segment, it manipulates the token
//! string directly rather than going through the encoder, for the same reason.

use tetherscript::jwtrs::claims::Claims;
use tetherscript::jwtrs::config::ValidationConfig;
use tetherscript::jwtrs::error::JwtError;
use tetherscript::jwtrs::error_claims::ClaimError;
use tetherscript::jwtrs::error_shape::ShapeError;
use tetherscript::jwtrs::limits::{MAX_ROLES, MAX_TOKEN_BYTES};
use tetherscript::jwtrs::test_verifier::StubVerifier;
use tetherscript::jwtrs::testdata::{keycloak_token, token_with};

/// The signature bytes the stub verifier accepts throughout.
const GOOD_SIG: &str = "sig-ok";

/// The realm every fixture claims to come from.
const ISSUER: &str = "https://sso.example/realms/main";

/// A config pinned to RS256, issuer `ISSUER`, audience `web-app`, default skew.
fn config() -> ValidationConfig {
    ValidationConfig::rs256(ISSUER, ["web-app"])
}

/// The stub that accepts exactly `GOOD_SIG`.
fn verifier() -> StubVerifier {
    StubVerifier::accepting(GOOD_SIG)
}

/// Validate at `now` against the standard config and the accepting stub.
fn validate_at(token: &str, now: i64) -> Result<Claims, JwtError> {
    Claims::validate(token, &config(), now, &verifier())
}

/// A payload with the standard registered claims plus `extra` members.
fn payload_with(extra: &str) -> String {
    format!(r#"{{"iss":"{ISSUER}","sub":"user-1","aud":"web-app","exp":1000{extra}}}"#)
}

/// The standard RS256 header.
const RS256_HEADER: &str = r#"{"alg":"RS256","kid":"key-a","typ":"JWT"}"#;

#[test]
fn accepts_a_valid_keycloak_token() {
    let claims = validate_at(&keycloak_token(1_000, GOOD_SIG), 950).expect("should validate");
    assert_eq!(claims.iss, ISSUER);
    assert_eq!(claims.sub, "user-1");
    assert_eq!(claims.aud, vec!["web-app".to_string()]);
    assert_eq!(claims.exp, 1_000);
    assert_eq!(claims.nbf, Some(700));
    assert_eq!(claims.iat, Some(700));
    assert_eq!(claims.azp.as_deref(), Some("web-app"));
    assert_eq!(claims.kid.as_deref(), Some("key-a"));
}

#[test]
fn rejects_alg_none_by_name() {
    // RFC 7515 §A.5's unsecured JWS. A verifier that dispatched on `alg` would
    // reach a "nothing to check" branch and accept an attacker-authored token.
    let token = token_with(r#"{"alg":"none"}"#, &payload_with(""), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::AlgNone))
    );
}

#[test]
fn rejects_alg_none_in_any_case() {
    let token = token_with(r#"{"alg":"NONE"}"#, &payload_with(""), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::AlgNone))
    );
}

#[test]
fn rejects_hs256_when_rs256_is_expected() {
    // Algorithm confusion: the RSA public key is public, so if the verifier read
    // `HS256` from the header and HMACed with the looked-up key material, the
    // attacker could compute that MAC too. Pinning refuses before any key is fetched.
    let token = token_with(
        r#"{"alg":"HS256","kid":"key-a"}"#,
        &payload_with(""),
        GOOD_SIG,
    );
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::AlgMismatch {
            got: "HS256".to_string(),
            expected: "RS256",
        }))
    );
}

#[test]
fn rejects_a_missing_alg() {
    let token = token_with(r#"{"kid":"key-a"}"#, &payload_with(""), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::MissingAlg))
    );
}

#[test]
fn rejects_a_non_string_alg() {
    let token = token_with(r#"{"alg":256}"#, &payload_with(""), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::AlgNotString("int")))
    );
}

#[test]
fn rejects_a_tampered_payload_at_the_signature() {
    // The payload gained an `"admin":true` member. The signing input is sliced from
    // the token, so a real RSA verifier would see different bytes and refuse; the
    // stub models that by refusing any signature other than `GOOD_SIG`, and the
    // tampered token carries `forged`.
    let tampered = token_with(RS256_HEADER, &payload_with(r#","admin":true"#), "forged");
    assert!(matches!(
        validate_at(&tampered, 950),
        Err(JwtError::Signature(_))
    ));
}

#[test]
fn returns_no_claims_when_the_signature_fails() {
    // The token is otherwise perfectly valid: right issuer, right audience, inside
    // its window, well-formed roles. The only defect is the signature.
    let token = keycloak_token(1_000, GOOD_SIG);
    let outcome = Claims::validate(&token, &config(), 950, &StubVerifier::rejecting());
    assert!(matches!(&outcome, Err(JwtError::Signature(_))));
    // There is no `Claims` value to inspect, and the error carries no claim data:
    // the subject of a rejected token cannot leak through this API.
    let message = outcome.expect_err("must fail").to_string();
    assert!(!message.contains("user-1"), "error leaked `sub`: {message}");
    assert!(!message.contains("admin"), "error leaked a role: {message}");
}

#[test]
fn claim_checks_do_not_run_before_the_signature_check() {
    // Expired, wrong issuer, wrong audience, and no `sub` — yet the reported failure
    // is the signature, because nothing reads the payload until verification passes.
    let token = token_with(
        RS256_HEADER,
        r#"{"iss":"evil","aud":"other","exp":1}"#,
        "forged",
    );
    assert!(matches!(
        validate_at(&token, 9_999),
        Err(JwtError::Signature(_))
    ));
}

#[test]
fn rejects_an_expired_token() {
    // 1_100 is 100s past `exp`, which is past the 60s default skew.
    assert_eq!(
        validate_at(&keycloak_token(1_000, GOOD_SIG), 1_100),
        Err(JwtError::Claim(ClaimError::Expired {
            exp: 1_000,
            now: 1_100,
            skew: 60,
        }))
    );
}

#[test]
fn accepts_a_just_expired_token_within_skew() {
    // 30s past `exp` but inside the 60s tolerance: ordinary clock drift, not a
    // stale credential.
    assert!(validate_at(&keycloak_token(1_000, GOOD_SIG), 1_030).is_ok());
}

#[test]
fn skew_is_symmetric_at_both_edges() {
    let token = keycloak_token(1_000, GOOD_SIG); // nbf == 700
                                                 // `exp + skew` is exclusive: exactly one second inside is accepted, the boundary
                                                 // itself is not.
    assert!(validate_at(&token, 1_059).is_ok());
    assert!(validate_at(&token, 1_060).is_err());
    // `nbf - skew` is inclusive on the same one number.
    assert!(validate_at(&token, 640).is_ok());
    assert!(validate_at(&token, 639).is_err());
}

#[test]
fn a_zero_skew_config_admits_no_drift() {
    let token = keycloak_token(1_000, GOOD_SIG);
    let strict = config().with_skew_secs(0);
    assert!(Claims::validate(&token, &strict, 999, &verifier()).is_ok());
    assert!(Claims::validate(&token, &strict, 1_000, &verifier()).is_err());
}

#[test]
fn rejects_an_nbf_in_the_future() {
    let token = token_with(RS256_HEADER, &payload_with(r#","nbf":900"#), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 500),
        Err(JwtError::Claim(ClaimError::NotYetValid {
            nbf: 900,
            now: 500,
            skew: 60,
        }))
    );
}

#[test]
fn rejects_a_token_with_no_exp() {
    // An absent `exp` is a permanent credential, and a stateless verifier has no
    // revocation list. So it is a missing claim, not "no limit".
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"user-1","aud":"web-app"}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::Missing("exp")))
    );
}

#[test]
fn rejects_a_non_numeric_exp() {
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"user-1","aud":"web-app","exp":"soon"}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::NotNumber {
            name: "exp",
            found: "str",
        }))
    );
}

#[test]
fn rejects_a_wrong_issuer() {
    let payload =
        r#"{"iss":"https://evil.example/realms/main","sub":"u","aud":"web-app","exp":1000}"#;
    let token = token_with(RS256_HEADER, payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::IssuerMismatch {
            got: "https://evil.example/realms/main".to_string(),
            expected: ISSUER.to_string(),
        }))
    );
}

#[test]
fn rejects_a_missing_issuer_as_missing_not_mismatched() {
    let token = token_with(
        RS256_HEADER,
        r#"{"sub":"u","aud":"web-app","exp":1000}"#,
        GOOD_SIG,
    );
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::Missing("iss")))
    );
}

#[test]
fn matches_aud_given_as_a_string() {
    let token = token_with(RS256_HEADER, &payload_with(""), GOOD_SIG);
    let claims = validate_at(&token, 950).expect("string aud should match");
    assert_eq!(claims.aud, vec!["web-app".to_string()]);
}

#[test]
fn matches_aud_given_as_an_array() {
    let payload =
        format!(r#"{{"iss":"{ISSUER}","sub":"u","aud":["account","web-app"],"exp":1000}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    let claims = validate_at(&token, 950).expect("array aud should match");
    assert_eq!(
        claims.aud,
        vec!["account".to_string(), "web-app".to_string()]
    );
}

#[test]
fn rejects_a_token_minted_for_another_audience() {
    // Confused deputy: this token is a genuine, correctly signed, unexpired token
    // from the right issuer — for a different service.
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"u","aud":"billing-api","exp":1000}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::AudienceMismatch {
            got: vec!["billing-api".to_string()],
            expected: vec!["web-app".to_string()],
        }))
    );
}

#[test]
fn rejects_an_absent_aud() {
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"u","exp":1000}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert!(matches!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::AudienceMismatch { .. }))
    ));
}

#[test]
fn rejects_a_non_string_aud() {
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"u","aud":7,"exp":1000}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::AudienceNotStringOrArray("int")))
    );
}

#[test]
fn rejects_an_aud_array_holding_a_non_string() {
    let payload = format!(r#"{{"iss":"{ISSUER}","sub":"u","aud":["web-app",7],"exp":1000}}"#);
    let token = token_with(RS256_HEADER, &payload, GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::AudienceNotStringOrArray("int")))
    );
}

#[test]
fn rejects_a_two_part_token() {
    let full = keycloak_token(1_000, GOOD_SIG);
    let two = full
        .rsplit_once('.')
        .expect("has three parts")
        .0
        .to_string();
    assert_eq!(
        validate_at(&two, 950),
        Err(JwtError::Shape(ShapeError::WrongSegmentCount(2)))
    );
}

#[test]
fn rejects_a_four_part_token() {
    let four = format!("{}.extra", keycloak_token(1_000, GOOD_SIG));
    assert_eq!(
        validate_at(&four, 950),
        Err(JwtError::Shape(ShapeError::WrongSegmentCount(4)))
    );
}

#[test]
fn rejects_a_one_part_token() {
    assert_eq!(
        validate_at("justonesegment", 950),
        Err(JwtError::Shape(ShapeError::WrongSegmentCount(1)))
    );
}

#[test]
fn rejects_an_empty_signature_segment() {
    // Three segments, but the third is empty: the unsecured form wearing a disguise.
    let full = keycloak_token(1_000, GOOD_SIG);
    let stripped = full.rsplit_once('.').expect("has three parts").0;
    let token = format!("{stripped}.");
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::EmptySegment("signature")))
    );
}

#[test]
fn rejects_standard_base64_input() {
    // `+` and `/` are the standard alphabet. No conforming JWS signer emits them, so
    // they are refused rather than translated — translating would give one token
    // several accepted encodings.
    let full = keycloak_token(1_000, GOOD_SIG);
    let (_, rest) = full.split_once('.').expect("has a header");
    for bad in ["ab+d", "ab/d", "abcd="] {
        let token = format!("{bad}.{rest}");
        match validate_at(&token, 950) {
            Err(JwtError::Shape(ShapeError::Base64 { segment, .. })) => {
                assert_eq!(segment, "header");
            }
            other => panic!("`{bad}` should be refused as base64, got {other:?}"),
        }
    }
}

#[test]
fn rejects_a_six_bit_remainder() {
    // A single leftover base64url character carries 6 bits, which is less than a
    // byte, so no byte string can encode to it.
    let full = keycloak_token(1_000, GOOD_SIG);
    let (_, rest) = full.split_once('.').expect("has a header");
    let token = format!("abcde.{rest}");
    assert!(matches!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::Base64 {
            segment: "header",
            ..
        }))
    ));
}

#[test]
fn rejects_a_non_object_payload() {
    // Each of these is valid JSON and an invalid JWT payload (RFC 7519 §3).
    for (json, found) in [
        ("[1,2,3]", "list"),
        ("7", "int"),
        ("null", "nil"),
        (r#""hi""#, "str"),
    ] {
        let token = token_with(RS256_HEADER, json, GOOD_SIG);
        assert_eq!(
            validate_at(&token, 950),
            Err(JwtError::Shape(ShapeError::NotAnObject {
                segment: "payload",
                found,
            })),
            "payload `{json}` should be refused"
        );
    }
}

#[test]
fn rejects_a_non_object_header() {
    let token = token_with("[1,2,3]", &payload_with(""), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::NotAnObject {
            segment: "header",
            found: "list",
        }))
    );
}

#[test]
fn rejects_a_malformed_json_payload() {
    let token = token_with(RS256_HEADER, r#"{"iss":"#, GOOD_SIG);
    assert!(matches!(
        validate_at(&token, 950),
        Err(JwtError::Shape(ShapeError::MalformedJson {
            segment: "payload",
            ..
        }))
    ));
}

#[test]
fn extracts_realm_and_resource_roles() {
    let claims = validate_at(&keycloak_token(1_000, GOOD_SIG), 950).expect("should validate");
    assert_eq!(
        claims.realm_roles,
        vec!["admin".to_string(), "offline_access".to_string()]
    );
    assert_eq!(
        claims.resource_roles,
        vec![("web-app".to_string(), vec!["viewer".to_string()])]
    );
    assert!(claims.has_realm_role("admin"));
    assert!(!claims.has_realm_role("superuser"));
    assert!(claims.has_resource_role("web-app", "viewer"));
    assert!(!claims.has_resource_role("web-app", "admin"));
    // Realm and resource scopes stay distinct: the realm's `admin` is not web-app's.
    assert!(!claims.has_resource_role("billing", "viewer"));
}

#[test]
fn extracts_multiple_resource_clients_in_sorted_order() {
    let extra = concat!(
        r#","resource_access":{"web-app":{"roles":["viewer"]},"#,
        r#""api":{"roles":["writer","reader"]}}"#
    );
    let token = token_with(RS256_HEADER, &payload_with(extra), GOOD_SIG);
    let claims = validate_at(&token, 950).expect("should validate");
    let names: Vec<&str> = claims
        .resource_roles
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    // Sorted, not JSON order: the in-tree parser's object order is unspecified.
    assert_eq!(names, vec!["api", "web-app"]);
    assert!(claims.has_resource_role("api", "writer"));
}

#[test]
fn a_token_with_no_role_containers_grants_nothing() {
    let token = token_with(RS256_HEADER, &payload_with(""), GOOD_SIG);
    let claims = validate_at(&token, 950).expect("should validate");
    assert!(claims.realm_roles.is_empty());
    assert!(claims.resource_roles.is_empty());
    assert!(!claims.has_realm_role("admin"));
}

#[test]
fn rejects_a_non_object_realm_access() {
    let token = token_with(
        RS256_HEADER,
        &payload_with(r#","realm_access":"admin""#),
        GOOD_SIG,
    );
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::RolesContainerNotObject {
            scope: "realm_access".to_string(),
            found: "str".to_string(),
        }))
    );
}

#[test]
fn rejects_a_non_array_roles_member() {
    let extra = r#","realm_access":{"roles":"admin"}"#;
    let token = token_with(RS256_HEADER, &payload_with(extra), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::RolesNotArray {
            scope: "realm_access".to_string(),
            found: "str".to_string(),
        }))
    );
}

#[test]
fn rejects_a_non_string_role() {
    // A numeric "role" is refused rather than stringified: `7` becoming `"7"` could
    // collide with a real role name.
    let extra = r#","realm_access":{"roles":["admin",7]}"#;
    let token = token_with(RS256_HEADER, &payload_with(extra), GOOD_SIG);
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::RoleNotString {
            scope: "realm_access".to_string(),
            found: "int".to_string(),
        }))
    );
}

#[test]
fn rejects_too_many_roles() {
    // 300 short role names: comfortably inside the 8 KiB token bound, so the role
    // bound is what fires rather than the size bound.
    let roles: Vec<String> = (0..300).map(|index| format!(r#""r{index}""#)).collect();
    let extra = format!(r#","realm_access":{{"roles":[{}]}}"#, roles.join(","));
    let token = token_with(RS256_HEADER, &payload_with(&extra), GOOD_SIG);
    assert!(
        token.len() < MAX_TOKEN_BYTES,
        "fixture must not trip the size bound"
    );
    assert_eq!(
        validate_at(&token, 950),
        Err(JwtError::Claim(ClaimError::TooManyRoles {
            scope: "realm_access".to_string(),
            count: 300,
            limit: MAX_ROLES,
        }))
    );
}

#[test]
fn rejects_an_oversized_token() {
    // A long-but-well-formed payload: refused on size before any decoding happens.
    let filler = "A".repeat(MAX_TOKEN_BYTES + 1);
    let token = format!("{}.{}", keycloak_token(1_000, GOOD_SIG), filler);
    match validate_at(&token, 950) {
        Err(JwtError::Shape(ShapeError::TokenTooLarge { bytes, limit })) => {
            assert_eq!(limit, MAX_TOKEN_BYTES);
            assert!(bytes > limit);
        }
        other => panic!("oversized token should be refused, got {other:?}"),
    }
}

#[test]
fn rejects_a_mismatched_typ_when_one_is_required() {
    let config = config().requiring_typ("JWT");
    let token = token_with(
        r#"{"alg":"RS256","typ":"at+jwt"}"#,
        &payload_with(""),
        GOOD_SIG,
    );
    assert_eq!(
        Claims::validate(&token, &config, 950, &verifier()),
        Err(JwtError::Shape(ShapeError::TypMismatch {
            got: "at+jwt".to_string(),
            expected: "JWT".to_string(),
        }))
    );
}

#[test]
fn every_error_message_names_the_failed_check() {
    let cases: Vec<JwtError> = vec![
        JwtError::Shape(ShapeError::AlgNone),
        JwtError::Shape(ShapeError::WrongSegmentCount(2)),
        JwtError::Signature("stub refused".to_string()),
        JwtError::Claim(ClaimError::Missing("exp")),
        JwtError::Claim(ClaimError::Expired {
            exp: 1,
            now: 2,
            skew: 60,
        }),
        JwtError::Claim(ClaimError::TooManyResourceClients {
            count: 99,
            limit: 64,
        }),
    ];
    for error in cases {
        let text = error.to_string();
        assert!(text.starts_with("jwtrs: "), "unprefixed: {text}");
        // "Error" is not an error message; every one must say something specific.
        assert!(text.len() > 20, "uninformative: {text}");
    }
}
