//! Integration tests for ad-blocking builtins.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn temp_file(label: &str, source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tetherscript-adblock-{label}-{}-{}.tether",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, source).expect("write temp");
    path
}

fn run(label: &str, source: &str) -> (bool, String, String) {
    let path = temp_file(label, source);
    let result = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "--interp", path.to_str().unwrap()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run tetherscript");
    let _ = std::fs::remove_file(&path);
    (
        result.status.success(),
        String::from_utf8_lossy(&result.stdout).to_string(),
        String::from_utf8_lossy(&result.stderr).to_string(),
    )
}

#[test]
fn parse_counts_network_and_cosmetic_rules() {
    let src = "fn main() {\n    let list = \"||ads.example.com^\\n! comment\\nexample.com##.ad\\n\"\n    let rules = adblock_parse(list)\n    println(adblock_rule_count(rules))\n}";
    let (ok, out, err) = run("parse", src);
    assert!(ok, "{err}");
    assert!(out.contains("2"), "expected 2, got: {out}");
}

#[test]
fn blocks_network_request_to_ad_domain() {
    let src = "fn main() {\n    let rules = adblock_parse(\"||ads.example.com^\")\n    let blocked = adblock_should_block(rules, \"https://ads.example.com/banner.gif\", \"site.com\")\n    println(blocked)\n}";
    let (ok, out, err) = run("block", src);
    assert!(ok, "{err}");
    assert!(out.contains("true"), "expected true, got: {out}");
}

#[test]
fn exception_overrides_block() {
    let src = "fn main() {\n    let rules = adblock_parse(\"||allowed.com^\\n@@||allowed.com^\")\n    let blocked = adblock_should_block(rules, \"https://allowed.com/img.png\", \"site.com\")\n    println(blocked)\n}";
    let (ok, out, err) = run("except", src);
    assert!(ok, "{err}");
    assert!(out.contains("false"), "expected false, got: {out}");
}

#[test]
fn cosmetic_selectors_returned_for_domain() {
    let src = "fn main() {\n    let rules = adblock_parse(\"example.com##.ad-banner\\nnews.com##.sidebar\")\n    let sels = adblock_cosmetic_selectors(rules, \"example.com\")\n    println(sels.len())\n    println(sels.contains(\".ad-banner\"))\n}";
    let (ok, out, err) = run("cosmetic", src);
    assert!(ok, "{err}");
    assert!(out.contains("1\ntrue"), "expected 1 selector, got: {out}");
}
