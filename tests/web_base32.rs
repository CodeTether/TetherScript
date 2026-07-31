//! Integration coverage for the base32 built-ins.
//!
//! Every expected encoding below is a published RFC 4648 section 10 test vector,
//! not a value produced by this implementation. That matters: a codec checked only
//! against its own output will happily agree with itself while being wrong, and a
//! wrong base32 secret surfaces as an authenticator code that never validates.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its trimmed stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_base32_{}", std::process::id()));
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

/// RFC 4648 section 10, the complete published vector set.
const VECTORS: [(&str, &str); 6] = [
    ("f", "MY======"),
    ("fo", "MZXQ===="),
    ("foo", "MZXW6==="),
    ("foob", "MZXW6YQ="),
    ("fooba", "MZXW6YTB"),
    ("foobar", "MZXW6YTBOI======"),
];

#[test]
fn encode_matches_every_rfc4648_vector() {
    for (input, expected) in VECTORS {
        let out = run(&format!(
            "fn main() {{ println(base32_encode(\"{input}\")) }}\n"
        ));
        assert_eq!(out, expected, "base32_encode({input:?})");
    }
}

#[test]
fn decode_matches_every_rfc4648_vector() {
    for (expected, encoded) in VECTORS {
        let out = run(&format!(
            "fn main() {{ println(base32_decode(\"{encoded}\").unwrap()) }}\n"
        ));
        assert_eq!(out, expected, "base32_decode({encoded:?})");
    }
}

/// The empty string is the one input that produces no characters at all.
#[test]
fn empty_input_round_trips() {
    let out = run(r#"
fn main() {
    println("[" + base32_encode("") + "]")
    println("[" + base32_decode("").unwrap() + "]")
}
"#);
    assert_eq!(out, "[]\n[]");
}

#[test]
fn nopad_output_drops_padding_but_keeps_characters() {
    let out = run(r#"
fn main() {
    println(base32_encode_nopad("f"))
    println(base32_encode_nopad("foo"))
    println(base32_encode_nopad("foobar"))
    println(base32_encode_nopad("fooba"))
}
"#);
    // Same significant characters as the padded vectors, without the `=` runs.
    assert_eq!(out, "MY\nMZXW6\nMZXW6YTBOI\nMZXW6YTB");
}

/// A secret copied from an authenticator app often arrives without padding.
#[test]
fn decode_accepts_unpadded_input() {
    let out = run(r#"
fn main() {
    println(base32_decode("MZXW6").unwrap())
    println(base32_decode("MZXW6YTBOI").unwrap())
}
"#);
    assert_eq!(out, "foo\nfoobar");
}

#[test]
fn decode_accepts_lower_case() {
    let out = run(r#"
fn main() {
    println(base32_decode("mzxw6ytboi======").unwrap())
    println(base32_decode("mZxW6===").unwrap())
}
"#);
    assert_eq!(out, "foobar\nfoo");
}

#[test]
fn round_trip_preserves_multi_byte_utf8() {
    let out = run(r#"
fn main() {
    let text = "café → ☕"
    println(str(base32_decode(base32_encode(text)).unwrap() == text))
}
"#);
    assert_eq!(out, "true");
}

/// `0`, `1`, and `8` are excluded from the alphabet, so they must be rejected by
/// name rather than silently skipped.
#[test]
fn decode_names_an_invalid_character_and_its_position() {
    let out = run(r#"
fn main() {
    let bad = base32_decode("MZXW0===")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains('0') && lines[1].contains("position 4"),
        "error must name the character and position, got: {}",
        lines[1]
    );
}

#[test]
fn decode_rejects_padding_in_the_middle() {
    let out = run(r#"
fn main() {
    let bad = base32_decode("MY======MY======")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("padding") && lines[1].contains("position 2"),
        "error must locate the stray padding, got: {}",
        lines[1]
    );
}

/// Remainders of 1, 3, and 6 characters cannot come from any input length.
#[test]
fn decode_rejects_an_impossible_length() {
    let out = run(r#"
fn main() {
    println(str(base32_decode("M").is_err()))
    println(str(base32_decode("MZX").is_err()))
    println(str(base32_decode("MZXW6Y").is_err()))
}
"#);
    assert_eq!(out, "true\ntrue\ntrue");
}

/// Two texts must never decode to the same bytes, so a non-canonical tail is an
/// error rather than something quietly masked off.
#[test]
fn decode_rejects_non_zero_unused_tail_bits() {
    let out = run(r#"
fn main() {
    let bad = base32_decode("MZ")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("unused bits"),
        "error must explain the non-canonical tail, got: {}",
        lines[1]
    );
}

#[test]
fn decode_rejects_excess_trailing_padding() {
    let out = run(r#"
fn main() {
    let bad = base32_decode("MY=======")
    println(str(bad.is_err()))
    println(bad.err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("padding") && lines[1].contains("multiple of 8"),
        "error must explain bad padding, got: {}",
        lines[1]
    );
}
