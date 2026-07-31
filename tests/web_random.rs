//! Integration coverage for the `random_*` built-ins.
//!
//! These assert *properties*, never specific outputs: a test that pinned a
//! literal value would either be wrong or prove the generator is not random.
//! Range, alphabet, length, distinctness, and the named error paths are checked
//! instead.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its stdout, failing the test on a non-zero exit.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_random_{}", std::process::id()));
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
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn bytes_hex_length_is_twice_the_byte_count() {
    let out = run(r#"
fn main() {
    for n in [1, 8, 16, 32] {
        println(str(random_bytes_hex(n).unwrap().len()))
    }
}
"#);
    assert_eq!(out.lines().collect::<Vec<_>>(), ["2", "16", "32", "64"]);
}

#[test]
fn bytes_hex_uses_only_lowercase_hex_digits() {
    let out = run(r#"
fn main() {
    let allowed = "0123456789abcdef"
    let value = random_bytes_hex(64).unwrap()
    let mut bad = 0
    // `split("")` yields one part per character, plus an empty part at each end.
    for ch in value.split("") {
        if ch != "" && !allowed.contains(ch) { bad = bad + 1 }
    }
    println(str(bad))
    println(str(value.len()))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "0", "unexpected characters in hex output: {out}");
    assert_eq!(lines[1], "128");
}

#[test]
fn successive_calls_return_different_values() {
    let out = run(r#"
fn main() {
    println(str(random_bytes_hex(32).unwrap() == random_bytes_hex(32).unwrap()))
    println(str(random_token(32).unwrap() == random_token(32).unwrap()))
}
"#);
    // A collision on 32 random bytes would be astronomically unlikely, so equal
    // values here mean cached state rather than bad luck.
    assert_eq!(out.lines().collect::<Vec<_>>(), ["false", "false"]);
}

#[test]
fn token_is_url_safe_and_unpadded() {
    let out = run(r#"
fn main() {
    let token = random_token(32).unwrap()
    println(str(token.contains("=")))
    println(str(token.contains("+")))
    println(str(token.contains("/")))
    println(str(token.len() > 0))
}
"#);
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false", "false", "true"],
        "token must be unpadded and URL-safe: {out}"
    );
}

#[test]
fn int_always_lands_inside_the_requested_range() {
    let out = run(r#"
fn main() {
    let mut outside = 0
    let mut i = 0
    while i < 300 {
        let value = random_int(10, 20).unwrap()
        if value < 10 || value >= 20 { outside = outside + 1 }
        i = i + 1
    }
    println(str(outside))
}
"#);
    assert_eq!(out.trim(), "0", "a draw escaped [10, 20): {out}");
}

#[test]
fn int_reaches_both_ends_of_a_tiny_range() {
    let out = run(r#"
fn main() {
    let mut low = false
    let mut high = false
    let mut i = 0
    while i < 200 {
        let value = random_int(0, 2).unwrap()
        if value == 0 { low = true }
        if value == 1 { high = true }
        i = i + 1
    }
    println(str(low))
    println(str(high))
}
"#);
    // Both outcomes in 200 draws; missing one would indicate a stuck generator.
    assert_eq!(out.lines().collect::<Vec<_>>(), ["true", "true"]);
}

#[test]
fn choice_returns_an_element_of_the_list() {
    let out = run(r#"
fn main() {
    let items = ["alpha", "beta", "gamma"]
    let mut foreign = 0
    let mut i = 0
    while i < 100 {
        let pick = random_choice(items).unwrap()
        if pick != "alpha" && pick != "beta" && pick != "gamma" { foreign = foreign + 1 }
        i = i + 1
    }
    println(str(foreign))
}
"#);
    assert_eq!(out.trim(), "0", "choice returned a foreign value: {out}");
}

#[test]
fn zero_and_negative_counts_are_named_errors() {
    let out = run(r#"
fn main() {
    println(random_bytes_hex(0).err())
    println(random_bytes_hex(-4).err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("positive"), "got: {}", lines[0]);
    assert!(lines[1].contains("positive"), "got: {}", lines[1]);
}

#[test]
fn oversized_counts_are_capped_with_a_named_error() {
    let out = run(r#"
fn main() {
    println(random_bytes_hex(100000).err())
    println(random_token(100000).err())
}
"#);
    for line in out.lines() {
        assert!(
            line.contains("at most"),
            "error should name the cap, got: {line}"
        );
    }
}

#[test]
fn an_empty_range_is_a_named_error() {
    let out = run(r#"
fn main() {
    println(random_int(5, 5).err())
    println(random_int(9, 3).err())
}
"#);
    for line in out.lines() {
        assert!(
            line.contains("less than"),
            "error should explain the bound order, got: {line}"
        );
    }
}

#[test]
fn an_empty_list_is_a_named_error() {
    let out = run(r#"
fn main() {
    println(random_choice([]).err())
}
"#);
    assert!(out.contains("empty"), "got: {out}");
}
