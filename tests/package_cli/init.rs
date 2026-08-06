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

#[test]
fn new_creates_runnable_package_project() {
    let root = support::root("package-new");
    let output = support::new_project(&root);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("tetherscript.json").is_file());
    assert!(root.join("src/main.tether").is_file());

    let run = support::command(&["run"], Some(&root));
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("Hello from tetherscript!"));
}

#[test]
fn new_requires_project_directory() {
    let output = support::command(&["new"], None);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected a project directory"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
