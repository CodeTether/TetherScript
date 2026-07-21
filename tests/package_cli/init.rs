use super::support;

#[test]
fn init_creates_manifest_and_entry_without_overwriting() {
    let root = support::root("package-init");
    let output = support::init(&root);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = std::fs::read_to_string(root.join("tetherscript.json")).unwrap();
    assert!(manifest.contains("\"entry\": \"src/main.tether\""));
    assert!(root.join("src/main.tether").is_file());

    let second = support::init(&root);
    assert!(!second.status.success());
    assert!(
        String::from_utf8_lossy(&second.stderr).contains("refusing to overwrite"),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
}
