//! Tests for script-declared authority headers.

use super::script;

#[test]
fn recognizes_agent_authority_header() {
    let src = "// tetherscript: authority agent\nfn main() {}";
    assert!(script::full_access(src, false));
}

#[test]
fn stops_scanning_after_code() {
    let src = "fn main() {}\n// tetherscript: authority agent";
    assert!(!script::full_access(src, false));
}

#[test]
fn preserves_cli_full_access() {
    assert!(script::full_access("fn main() {}", true));
}

#[test]
fn recognizes_hot_reload_header() {
    let src = "// tetherscript: hot-reload\nfn main() {}";
    assert!(script::hot_reload(src));
}

#[test]
fn ignores_late_hot_reload_header() {
    let src = "fn main() {}\n// tetherscript: hot-reload";
    assert!(!script::hot_reload(src));
}
