//! Behaviour tests for the JWT built-ins.
//!
//! Every token used here is either produced by `jwt_sign` or assembled in-script
//! from `hex_encode`/`hmac_sha256_hex` primitives, so no expected value is an
//! invented signature. The forged tokens (`alg: none`, swapped `alg`, tampered
//! payload) are built by re-encoding a real header or payload, which is exactly
//! how an attacker would produce them.
//!
//! These drive the built-ins through the interpreter, since that is the surface
//! scripts actually see.

use std::process::Command;

/// Run a tetherscript program from source text and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "tether_web_jwt_{}_{:?}",
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

#[test]
fn sign_then_verify_round_trips_claims() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    claims.roles = ["admin"]
    let token = jwt_sign(claims, "shared-secret").unwrap()
    let back = jwt_verify(token, "shared-secret").unwrap()
    println(back.sub)
    println(back.roles.join(","))
}"#);
    assert_eq!(out, "user-1\nadmin");
}

/// A JWS segment must never carry base64 padding or the standard alphabet.
#[test]
fn token_is_unpadded_base64url() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "padding-check-aaaa"
    let token = jwt_sign(claims, "k").unwrap()
    println(str(token.contains("=")))
    println(str(token.contains("+")))
    println(str(token.contains("/")))
    println(str(token.split(".").len()))
}"#);
    assert_eq!(out, "false\nfalse\nfalse\n3");
}

#[test]
fn verification_fails_under_a_wrong_secret() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    let token = jwt_sign(claims, "right-secret").unwrap()
    let r = jwt_verify(token, "wrong-secret")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("signature does not match"), "got: {out}");
}

/// The `alg: none` unsecured JWS must be refused even with a valid-looking body.
#[test]
fn alg_none_token_is_rejected() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "attacker"
    let real = jwt_sign(claims, "k").unwrap()
    let parts = real.split(".")

    // Unpadded base64url of {"alg":"none","typ":"JWT"}, keeping the real payload.
    let forged = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0." + parts[1] + "." + parts[2]
    let r = jwt_verify(forged, "k")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("alg"), "error should name alg, got: {out}");
}

#[test]
fn hs256_token_with_swapped_alg_is_rejected() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    let real = jwt_sign(claims, "k").unwrap()
    let parts = real.split(".")
    // RS256 header, unpadded base64url of {"alg":"RS256","typ":"JWT"}.
    let forged = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9." + parts[1] + "." + parts[2]
    let r = jwt_verify(forged, "k")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(
        out.contains("RS256"),
        "error should name the alg, got: {out}"
    );
}

#[test]
fn tampered_payload_is_rejected() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    let real = jwt_sign(claims, "k").unwrap()
    let parts = real.split(".")

    let elevated = map()
    elevated.sub = "admin"
    let other = jwt_sign(elevated, "k").unwrap()
    let swapped = other.split(".")

    // Real header and signature, attacker's payload.
    let forged = parts[0] + "." + swapped[1] + "." + parts[2]
    let r = jwt_verify(forged, "k")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("signature does not match"), "got: {out}");
}

#[test]
fn expired_exp_is_rejected() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    claims.exp = (time_now_ms() / 1000) - 60
    let token = jwt_sign(claims, "k").unwrap()
    let r = jwt_verify(token, "k")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("expired"), "got: {out}");
}

#[test]
fn future_nbf_is_rejected() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    claims.nbf = (time_now_ms() / 1000) + 3600
    let token = jwt_sign(claims, "k").unwrap()
    let r = jwt_verify(token, "k")
    println(str(r.is_err()))
    println(r.err())
}"#);
    assert!(out.starts_with("true\n"), "got: {out}");
    assert!(out.contains("not yet valid"), "got: {out}");
}

/// A valid `exp`/`nbf` window must still verify.
#[test]
fn token_inside_its_validity_window_verifies() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    claims.nbf = (time_now_ms() / 1000) - 60
    claims.exp = (time_now_ms() / 1000) + 3600
    let token = jwt_sign(claims, "k").unwrap()
    println(jwt_verify(token, "k").unwrap().sub)
}"#);
    assert_eq!(out, "user-1");
}

#[test]
fn wrong_segment_count_is_named() {
    let out = run(r#"fn main() {
    println(jwt_verify("only.two", "k").err())
    println(jwt_verify("a.b.c.d", "k").err())
}"#);
    assert!(out.contains("3 dot-separated segments"), "got: {out}");
}

#[test]
fn bad_base64_and_bad_json_are_named() {
    let out = run(r#"fn main() {
    // `!` is outside the base64url alphabet.
    println(jwt_verify("aGVhZGVy!.cGF5bG9hZA.c2ln", "k").err())
    // Valid base64url of "notjson", which is not JSON at all.
    println(jwt_decode_unverified("aGVhZGVy.bm90anNvbg.c2ln").err())
    println(jwt_verify("bm90anNvbg.cGF5bG9hZA.c2ln", "k").err())
}"#);
    assert!(out.contains("base64url"), "got: {out}");
    assert!(out.contains("not valid JSON"), "got: {out}");
}

/// `jwt_decode_unverified` must read claims without checking the signature.
#[test]
fn decode_unverified_ignores_signature_and_expiry() {
    let out = run(r#"fn main() {
    let claims = map()
    claims.sub = "user-1"
    claims.exp = (time_now_ms() / 1000) - 60
    let token = jwt_sign(claims, "real-secret").unwrap()

    // Expired and signed with a different secret, yet still decodable.
    println(jwt_decode_unverified(token).unwrap().sub)
    println(str(jwt_verify(token, "real-secret").is_err()))
}"#);
    assert_eq!(out, "user-1\ntrue");
}

#[test]
fn non_string_arguments_name_the_parameter() {
    let out = run(r#"fn main() {
    println(jwt_sign(map(), 1).err())
    println(jwt_verify(1, "k").err())
    println(jwt_sign("not-a-map", "k").err())
}"#);
    assert!(out.contains("jwt_sign: secret"), "got: {out}");
    assert!(out.contains("jwt_verify: token"), "got: {out}");
    assert!(out.contains("claims must be map"), "got: {out}");
}
