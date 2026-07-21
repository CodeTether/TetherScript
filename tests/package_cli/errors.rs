use super::support;

#[test]
fn malformed_manifest_names_the_invalid_field() {
    let root = support::root("package-error");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("tetherscript.json"),
        r#"{"schema":1,"package":{"name":"demo","version":"0.1.0","entry":7}}"#,
    )
    .unwrap();

    let output = support::command(&["run", root.to_str().unwrap()], None);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("package.entry must be a string"),
        "{stderr}"
    );
}

#[test]
fn missing_manifest_is_reported() {
    let root = support::root("package-missing");
    std::fs::create_dir_all(&root).unwrap();
    let output = support::command(&["run", root.to_str().unwrap()], None);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tetherscript.json"), "{stderr}");
}
