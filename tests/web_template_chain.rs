//! Multi-level inheritance and its failure modes.
//!
//! Split from `web_template_inherit.rs` so the single-parent cases and the chain
//! cases stay separate.

use std::process::Command;

/// Run a script, returning trimmed stdout.
fn stdout_of(source: &str) -> String {
    static CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case = CASE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_tpl_chain_{}_{case}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("case.tether");
    std::fs::write(&path, source).expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        output.status.success(),
        "script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// A three-level hierarchy: root defines two blocks, middle overrides one.
///
/// `accessor` is appended to the render call, so a test can read `.unwrap()` for a
/// success or `.err()` for a failure.
fn chain_script(leaf: &str, accessor: &str) -> String {
    String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    let t = map()\n"
        + "    t[\"root\"] = \"R:\\{% block a %\\}root-a\\{% endblock %\\}"
        + "/\\{% block b %\\}root-b\\{% endblock %\\}\"\n"
        + "    t[\"middle\"] = \"\\{% extends \\\"root\\\" %\\}"
        + "\\{% block a %\\}mid-a\\{% endblock %\\}\"\n"
        + "    println(template_render_inherited(\""
        + leaf
        + "\", c, t)"
        + accessor
        + ")\n}\n"
}

/// The most-derived template must win at every level.
#[test]
fn the_leaf_overrides_the_middle_which_overrides_the_root() {
    let leaf = "\\{% extends \\\"middle\\\" %\\}\\{% block b %\\}leaf-b\\{% endblock %\\}";
    assert_eq!(
        stdout_of(&chain_script(leaf, ".unwrap()")),
        "R:mid-a/leaf-b"
    );
}

/// A block the leaf leaves alone must still show the middle's override, not the
/// root's default — otherwise the chain would only be one level deep.
#[test]
fn a_middle_override_survives_when_the_leaf_is_silent() {
    let leaf = "\\{% extends \\\"middle\\\" %\\}";
    assert_eq!(
        stdout_of(&chain_script(leaf, ".unwrap()")),
        "R:mid-a/root-b"
    );
}

/// A leaf may override a block the middle never mentions.
#[test]
fn the_leaf_may_override_a_block_the_middle_skipped() {
    let leaf = "\\{% extends \\\"middle\\\" %\\}\\{% block a %\\}leaf-a\\{% endblock %\\}";
    assert_eq!(
        stdout_of(&chain_script(leaf, ".unwrap()")),
        "R:leaf-a/root-b"
    );
}

/// A missing parent must name the template: rendering a blank page instead would
/// ship a silently broken view.
#[test]
fn an_unknown_parent_names_the_template() {
    let leaf = "\\{% extends \\\"absent\\\" %\\}";
    let out = stdout_of(&chain_script(leaf, ".err()"));
    assert!(out.contains("absent"), "got: {out}");
}

/// A cycle must be reported rather than looping until memory runs out.
#[test]
fn a_cyclic_chain_is_reported() {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    let t = map()\n"
        + "    t[\"a\"] = \"\\{% extends \\\"b\\\" %\\}\"\n"
        + "    t[\"b\"] = \"\\{% extends \\\"a\\\" %\\}\"\n"
        + "    println(template_render_inherited(\"\\{% extends \\\"a\\\" %\\}\", c, t).err())\n}\n";
    let out = stdout_of(&script);
    assert!(out.contains("cyclic"), "got: {out}");
}

/// `extends` without a template map is a caller mistake worth naming precisely.
#[test]
fn extending_without_supplying_templates_is_an_error() {
    let script = String::from("fn main() {\n")
        + "    let c = map()\n"
        + "    println(template_render(\"\\{% extends \\\"base\\\" %\\}\", c).err())\n}\n";
    let out = stdout_of(&script);
    assert!(out.contains("template_render_inherited"), "got: {out}");
}
