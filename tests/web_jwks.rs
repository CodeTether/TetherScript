//! Behaviour tests for the JWK/JWKS built-ins.
//!
//! # Where the fixtures come from
//!
//! The two 2048-bit moduli and the 1024-bit one are the real moduli of RSA keys
//! generated with `openssl genrsa`, base64url-encoded, so the length and
//! bit-length assertions are about genuine key material rather than a hand-made
//! byte pattern. The exponent `AQAB` is the standard 65537. Every header and
//! signature segment below is real unpadded base64url of the JSON shown in the
//! comment beside it, which is exactly how a forger would assemble one.
//!
//! # What is deliberately *not* tested here
//!
//! Signature verification. This group produces key material and signing input
//! and performs no RSA arithmetic, so there is no "valid signature" case to
//! assert — only the refusals that must happen before any verifier runs.
//!
//! These drive the built-ins through the interpreter, since that is the surface
//! scripts actually see.

use std::process::Command;

/// Run a tetherscript program from source text and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "tether_web_jwks_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join("case.tether");
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
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// Real 2048-bit RSA modulus, base64url, 256 bytes decoded.
const N_A: &str = "qvja5RjDbB2pwZV_l4BVDJ1dX2K7DicMN6ApVfH0xHc1jsWtkZ7To2UkEZmS8w9bfKDdb8ShZbB8hxYAEtg_iZ-S6BnfSClTRAUMJXpCtTtWWndWaexW2muQxoKE-Rgt77UIhN2H3JLcvsg3zct58MULnIWedUrvpGd6cAlVz-jpkZZpYHVH7TWi1CzGicZ2i-PkOyiJpRmWuBmdfRcO2tBgQjEGhZA5A0oddbyPab26SYsSlDN--MGMrXYby5rQ6FWtQW3UxxtKxbQDPi9LK_RTYANq9wLIzjn08m9Hgvz8ZvRYJiThUQCbB3d0vL-HITzh8XLcjH-iNSuLxBbeAQ";

/// A second, distinct 2048-bit modulus, so `kid` selection has something to pick
/// between rather than trivially returning the only entry.
const N_B: &str = "nhsw2exHaE_olwxEhz_rRBOU-IP-OjNn_BFS-zgBXyFHM6DS5uIK6sZzNSZC-5eX_HNtWBtHztd_RxAXLAnFlf81wyYCBYTlZfe0UJHG7x2hk7Ku5-rixrShScVlkxoNrtmCaN_cd_vMpYXifN28pDTU0BfX35rC1aeroyj-kxPJwNUaGugu1Ld1qEoxfZ8i5TeOxCsbxEGQGXuY3jyR_NmMTXmuwIL6XZmK7tnToA3wmC532SHBziQpcaVQBfIEqfIa_E749k2hmSbnosWEurbnkb6aGqD4wKA57Hr1-pjNVscojYUo6kEELfVHSRV0jzlId9vlom1hQXxxsnQ9IQ";

/// Real 1024-bit modulus: well-formed base64url, but below the 2048-bit floor.
const N_1024: &str = "z39uXMGlByKOBTEVdfa6jHV2nfsySzJNTDvYK49BI9EFzVBhXDDmd8dkc2CpQ6F_Aw1csi-ChaLuzSQNcjgVS7pVZH0JSg4inbmsBXKuZTk0jItiPzeHEOSYrixPSDYwwlywTlsCEYXS2xc4Zr7FAt9pNpu4yjfWxKkNkDFJW1s";

/// Header of `{"alg":"RS256","typ":"JWT","kid":"key-a"}`.
const H_RS256_A: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS1hIn0";
/// Header of `{"alg":"RS384","typ":"JWT"}` — no `kid`, as single-key issuers emit.
const H_RS384_NOKID: &str = "eyJhbGciOiJSUzM4NCIsInR5cCI6IkpXVCJ9";
/// Header of `{"alg":"none","typ":"JWT","kid":"key-a"}` — the unsecured JWS.
const H_NONE: &str = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIiwia2lkIjoia2V5LWEifQ";
/// Header of `{"alg":"HS256","typ":"JWT","kid":"key-a"}` — the confusion attack.
const H_HS256: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS1hIn0";
/// Header of `{"typ":"JWT","kid":"key-a"}` — `alg` absent entirely.
const H_NOALG: &str = "eyJ0eXAiOiJKV1QiLCJraWQiOiJrZXktYSJ9";
/// Truncated JSON `{"alg": ` — valid base64url, invalid JSON.
const H_BADJSON: &str = "eyJhbGciOiA";
/// JSON array `["RS256"]` — valid JSON, but not an object.
const H_ARRAY: &str = "WyJSUzI1NiJd";
/// Payload of `{"sub":"user-1","exp":9999999999}`.
const PAYLOAD: &str = "eyJzdWIiOiJ1c2VyLTEiLCJleHAiOjk5OTk5OTk5OTl9";
/// 256 bytes of 0xAB, base64url: a syntactically plausible RS256 signature.
const SIG: &str = "q6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6urqw";

/// Build a tetherscript `certs()` returning a Keycloak-shaped two-key document.
///
/// The braces and quotes are escaped because tetherscript strings use `{` for
/// interpolation, so a literal JSON object must spell it `\{`.
fn certs_fn() -> String {
    format!(
        "fn certs() {{\n    \"\\{{\\\"keys\\\":[\
{},{}\
]\\}}\"\n}}\n",
        key_json("key-a", "RS256", "sig", N_A),
        key_json("key-b", "RS256", "sig", N_B)
    )
}

/// One escaped JWK object for embedding in a tetherscript string literal.
fn key_json(kid: &str, alg: &str, usage: &str, n: &str) -> String {
    format!(
        "\\{{\\\"kid\\\":\\\"{kid}\\\",\\\"kty\\\":\\\"RSA\\\",\\\"alg\\\":\\\"{alg}\\\",\
\\\"use\\\":\\\"{usage}\\\",\\\"n\\\":\\\"{n}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
    )
}

/// Wrap a JWKS document body (already escaped) in a `certs()` function.
fn certs_fn_raw(body: &str) -> String {
    format!("fn certs() {{\n    \"{body}\"\n}}\n")
}

/// A one-key document whose single key is described by the escaped JSON given.
fn one_key_doc(key: &str) -> String {
    certs_fn_raw(&format!("\\{{\\\"keys\\\":[{key}]\\}}"))
}

/// Escaped JWK object for the 1024-bit key, reused by two refusal tests.
fn weak_key_json() -> String {
    format!(
        "\\{{\\\"kid\\\":\\\"weak\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{N_1024}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
    )
}

// -- jwks_parse: the happy path -------------------------------------------------

#[test]
fn parses_a_two_key_keycloak_shaped_document() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let keys = jwks_parse(certs()).unwrap()
    println(str(keys.len()))
    println(keys[0].kid)
    println(keys[1].kid)
    println(keys[0].kty)
    println(keys[0].alg)
    println(keys[0].use)
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "2\nkey-a\nkey-b\nRSA\nRS256\nsig");
}

/// The raw bytes are what an RSA verifier needs, so their size is load-bearing.
#[test]
fn exposes_raw_modulus_and_exponent_bytes() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let key = jwks_parse(certs()).unwrap()[0]
    println(str(key.modulus.len()))
    println(str(key.modulus_bits))
    println(key.exponent.hex())
    println(str(key.exponent.len()))
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "256\n2048\n010001\n3");
}

/// `n`/`e` are aliases of `modulus`/`exponent`, so a script may use either
/// spelling and can never read two different values for one key.
#[test]
fn jwk_spelled_aliases_match_the_descriptive_names() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let key = jwks_parse(certs()).unwrap()[0]
    println(str(key.n.hex() == key.modulus.hex()))
    println(str(key.e.hex() == key.exponent.hex()))
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "true\ntrue");
}

/// An issuer that has revoked every key publishes an empty array; that is a
/// state, not a malformed document.
#[test]
fn an_empty_key_set_parses_to_an_empty_list() {
    let out = run(&format!(
        r#"{}
fn main() {{
    println(str(jwks_parse(certs()).unwrap().len()))
}}"#,
        certs_fn_raw("\\{\\\"keys\\\":[]\\}")
    ));
    assert_eq!(out, "0");
}

/// A key with no `alg` or `use` still parses; the fields read as nil rather than
/// being absent, so `key.alg` never surprises a caller.
#[test]
fn optional_alg_and_use_read_as_nil() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let key = jwks_parse(certs()).unwrap()[0]
    println(str(key.alg == nil))
    println(str(key.use == nil))
    println(key.kid)
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kid\\\":\\\"solo\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{N_A}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
        ))
    ));
    assert_eq!(out, "true\ntrue\nsolo");
}

// -- jwks_find: selection -------------------------------------------------------

#[test]
fn kid_selection_returns_the_matching_key() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let keys = jwks_parse(certs()).unwrap()
    println(jwks_find(keys, "key-b").unwrap().kid)
    println(str(jwks_find(keys, "key-b").unwrap().modulus.hex() == keys[1].modulus.hex()))
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "key-b\ntrue");
}

/// A miss must be an error, never a silent fallback to "some other key".
#[test]
fn kid_selection_miss_is_a_named_error() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_find(jwks_parse(certs()).unwrap(), "rotated-out")
    println(str(r.is_err()))
    println(r.err())
}}"#,
        certs_fn()
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("rotated-out"), "should name the kid: {out}");
    assert!(
        out.contains("key-a") && out.contains("key-b"),
        "should list what was available: {out}"
    );
}

#[test]
fn jwks_find_rejects_a_non_list_first_argument() {
    let out = run(r#"fn main() {
    let r = jwks_find("not a list", "key-a")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("must be a list"), "got: {out}");
}

// -- jwks_parse: key-material refusals -----------------------------------------

/// A symmetric `oct` entry in a JWKS document is how an attacker tries to get an
/// HMAC secret treated as an RSA public key.
#[test]
fn a_symmetric_oct_key_is_rejected() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        one_key_doc(
            "\\{\\\"kid\\\":\\\"sym\\\",\\\"kty\\\":\\\"oct\\\",\\\"k\\\":\\\"c2VjcmV0\\\"\\}"
        )
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("oct"), "should name the kty: {out}");
    assert!(out.contains("RSA"), "should say what is accepted: {out}");
}

#[test]
fn a_1024_bit_modulus_is_rejected() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kid\\\":\\\"weak\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{N_1024}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
        ))
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("128"), "should name the actual size: {out}");
    assert!(out.contains("2048"), "should name the floor: {out}");
}

/// An empty exponent decodes to zero, and exponentiation by zero returns 1 for
/// every input, i.e. a key that would "verify" anything.
#[test]
fn an_empty_exponent_is_rejected() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kid\\\":\\\"noexp\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{N_A}\\\",\\\"e\\\":\\\"\\\"\\}}"
        ))
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("exponent"), "got: {out}");
}

/// Selection depends on `kid`, so a key without one is a document bug, not a
/// key this group can quietly accept.
#[test]
fn a_key_without_a_kid_is_rejected() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{N_A}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
        ))
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("kid"), "got: {out}");
}

/// One bad key poisons the document rather than being pruned: a silently dropped
/// key is indistinguishable from a rotated-out one, and only one is safe.
#[test]
fn one_invalid_key_rejects_the_whole_document() {
    let body = format!(
        "\\{{\\\"keys\\\":[{},{}]\\}}",
        key_json("key-a", "RS256", "sig", N_A),
        weak_key_json()
    );
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        certs_fn_raw(&body)
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("keys[1]"), "should locate the key: {out}");
}

// -- jwks_parse: document shape ------------------------------------------------

#[test]
fn a_non_json_document_is_rejected() {
    let out = run(r#"fn main() {
    let r = jwks_parse("<html>not a jwks</html>")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("not valid JSON"), "got: {out}");
}

#[test]
fn a_document_without_keys_is_rejected() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        certs_fn_raw("\\{\\\"issuer\\\":\\\"https://example.test\\\"\\}")
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("keys"), "got: {out}");
}

#[test]
fn jwks_parse_rejects_a_non_string_argument() {
    let out = run(r#"fn main() {
    let r = jwks_parse(42)
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("must be str"), "got: {out}");
}

// -- base64url strictness ------------------------------------------------------

/// Standard base64 `+` must be refused rather than translated: one key must have
/// exactly one spelling.
#[test]
fn a_modulus_using_standard_base64_plus_is_rejected() {
    let mangled = format!("+{}", &N_A[1..]);
    let out = run(&format!(
        r#"{}
fn main() {{
    let r = jwks_parse(certs())
    println(str(r.is_err()))
    println(r.err())
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kid\\\":\\\"plus\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{mangled}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
        ))
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(
        out.contains("standard base64"),
        "should explain the alphabet: {out}"
    );
}

#[test]
fn a_token_signature_using_plus_is_rejected() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_rs256_parts("{H_RS256_A}.{PAYLOAD}.+{}")
    println(str(r.is_err()))
    println(r.err())
}}"#,
        &SIG[1..]
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("standard base64"), "got: {out}");
}

#[test]
fn a_modulus_using_slash_is_rejected() {
    let mangled = format!("/{}", &N_A[1..]);
    let out = run(&format!(
        r#"{}
fn main() {{
    println(jwks_parse(certs()).err())
}}"#,
        one_key_doc(&format!(
            "\\{{\\\"kid\\\":\\\"slash\\\",\\\"kty\\\":\\\"RSA\\\",\\\"n\\\":\\\"{mangled}\\\",\\\"e\\\":\\\"AQAB\\\"\\}}"
        ))
    ));
    assert!(out.contains("standard base64"), "got: {out}");
}

#[test]
fn a_padded_segment_is_rejected() {
    let out = run(&format!(
        r#"fn main() {{
    println(jwt_header("{H_RS256_A}=.{PAYLOAD}.{SIG}").err())
}}"#
    ));
    assert!(out.contains("unpadded"), "got: {out}");
}

// -- jwt_header: unverified, header only ---------------------------------------

/// The point of `jwt_header` is reading `kid` in order to select a key.
#[test]
fn jwt_header_exposes_kid_for_key_selection() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let header = jwt_header("{H_RS256_A}.{PAYLOAD}.{SIG}").unwrap()
    println(header.kid)
    println(header.alg)
    println(header.typ)
    println(jwks_find(jwks_parse(certs()).unwrap(), header.kid).unwrap().kid)
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "key-a\nRS256\nJWT\nkey-a");
}

/// Verifying nothing is the documented contract, and this test pins it: a token
/// whose signature is plainly wrong still yields a header. That is exactly why
/// the output must never inform an authorization decision.
#[test]
fn jwt_header_does_not_verify_the_signature() {
    let out = run(&format!(
        r#"fn main() {{
    let header = jwt_header("{H_RS256_A}.{PAYLOAD}.AAAA").unwrap()
    println(header.alg)
    println(header.kid)
}}"#
    ));
    assert_eq!(out, "RS256\nkey-a");
}

/// `alg: none` reaches `jwt_header` intact — reading a header is not a decision,
/// so refusing here would only push callers toward hand-rolled decoding. The
/// refusal belongs in `jwt_rs256_parts`, which is asserted below.
#[test]
fn jwt_header_reads_an_alg_none_header_without_judging_it() {
    let out = run(&format!(
        r#"fn main() {{
    println(jwt_header("{H_NONE}.{PAYLOAD}.{SIG}").unwrap().alg)
}}"#
    ));
    assert_eq!(out, "none");
}

#[test]
fn a_two_segment_token_is_rejected() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_header("{H_RS256_A}.{PAYLOAD}")
    println(str(r.is_err()))
    println(r.err())
}}"#
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("3 dot-separated"), "got: {out}");
    assert!(out.contains("got 2"), "should report the count: {out}");
}

#[test]
fn a_two_segment_token_is_rejected_by_rs256_parts_too() {
    let out = run(&format!(
        r#"fn main() {{
    println(jwt_rs256_parts("{H_RS256_A}.{PAYLOAD}").err())
}}"#
    ));
    assert!(out.contains("3 dot-separated"), "got: {out}");
}

#[test]
fn an_empty_signature_segment_is_rejected() {
    let out = run(&format!(
        r#"fn main() {{
    println(jwt_rs256_parts("{H_RS256_A}.{PAYLOAD}.").err())
}}"#
    ));
    assert!(out.contains("empty segment"), "got: {out}");
}

#[test]
fn a_malformed_header_json_is_rejected_with_a_named_error() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_header("{H_BADJSON}.{PAYLOAD}.{SIG}")
    println(str(r.is_err()))
    println(r.err())
}}"#
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(
        out.contains("jwt_header") && out.contains("header is not valid JSON"),
        "error must name the builtin and the segment: {out}"
    );
}

/// A header that is valid JSON but not an object has no members to read, so it
/// must be refused rather than silently treated as an empty header.
#[test]
fn a_header_that_is_a_json_array_is_rejected() {
    let out = run(&format!(
        r#"fn main() {{
    println(jwt_header("{H_ARRAY}.{PAYLOAD}.{SIG}").err())
}}"#
    ));
    assert!(out.contains("must be a JSON object"), "got: {out}");
}

#[test]
fn jwt_header_rejects_a_non_string_argument() {
    let out = run(r#"fn main() {
    println(jwt_header(nil).err())
}"#);
    assert!(out.contains("must be str"), "got: {out}");
}

// -- jwt_rs256_parts: algorithm policy and signing material -------------------

#[test]
fn rs256_parts_exposes_signing_input_signature_and_alg() {
    let out = run(&format!(
        r#"fn main() {{
    let parts = jwt_rs256_parts("{H_RS256_A}.{PAYLOAD}.{SIG}").unwrap()
    println(parts.alg)
    println(parts.kid)
    println(str(parts.signature.len()))
    println(parts.signing_input.to_string())
}}"#
    ));
    assert_eq!(
        out,
        format!("RS256\nkey-a\n256\n{H_RS256_A}.{PAYLOAD}"),
        "signing input must be exactly header.payload"
    );
}

/// RFC 7515 signs the ASCII of `header.payload`, so the third segment must not
/// leak into the signing input.
#[test]
fn signing_input_excludes_the_signature_segment() {
    let out = run(&format!(
        r#"fn main() {{
    let parts = jwt_rs256_parts("{H_RS256_A}.{PAYLOAD}.{SIG}").unwrap()
    println(str(parts.signing_input.to_string().contains("{SIG}")))
    println(str(parts.signing_input.to_string().split(".").len()))
}}"#
    ));
    assert_eq!(out, "false\n2");
}

#[test]
fn rs384_is_accepted_and_a_missing_kid_reads_as_nil() {
    let out = run(&format!(
        r#"fn main() {{
    let parts = jwt_rs256_parts("{H_RS384_NOKID}.{PAYLOAD}.{SIG}").unwrap()
    println(parts.alg)
    println(str(parts.kid == nil))
}}"#
    ));
    assert_eq!(out, "RS384\ntrue");
}

/// The unsecured JWS. Accepting it means accepting unsigned tokens.
#[test]
fn alg_none_is_rejected_by_rs256_parts() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_rs256_parts("{H_NONE}.{PAYLOAD}.{SIG}")
    println(str(r.is_err()))
    println(r.err())
}}"#
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("none"), "error should name the alg: {out}");
    assert!(
        out.contains("never accepted"),
        "error should be unambiguous: {out}"
    );
}

/// The RS256-to-HS256 confusion attack: the attacker re-signs with HMAC using the
/// published public key as the shared secret. Refusing the name closes it.
#[test]
fn a_symmetric_alg_is_rejected_by_rs256_parts() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_rs256_parts("{H_HS256}.{PAYLOAD}.{SIG}")
    println(str(r.is_err()))
    println(r.err())
}}"#
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("HS256"), "got: {out}");
    assert!(
        out.contains("RS256") && out.contains("RS384") && out.contains("RS512"),
        "error should list the accepted set: {out}"
    );
}

#[test]
fn a_header_without_alg_is_rejected_by_rs256_parts() {
    let out = run(&format!(
        r#"fn main() {{
    let r = jwt_rs256_parts("{H_NOALG}.{PAYLOAD}.{SIG}")
    println(str(r.is_err()))
    println(r.err())
}}"#
    ));
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("alg"), "got: {out}");
}

/// The end-to-end shape a caller uses: read `kid` unverified, select a trusted
/// key, then collect the bytes a verifier needs. No verification happens here.
#[test]
fn key_selection_then_parts_extraction_composes() {
    let out = run(&format!(
        r#"{}
fn main() {{
    let token = "{H_RS256_A}.{PAYLOAD}.{SIG}"
    let header = jwt_header(token).unwrap()
    let key = jwks_find(jwks_parse(certs()).unwrap(), header.kid).unwrap()
    let parts = jwt_rs256_parts(token).unwrap()
    println(key.kid)
    println(str(key.modulus.len()))
    println(parts.alg)
    println(str(parts.signature.len()))
    println(str(parts.signing_input.len() > 0))
}}"#,
        certs_fn()
    ));
    assert_eq!(out, "key-a\n256\nRS256\n256\ntrue");
}
