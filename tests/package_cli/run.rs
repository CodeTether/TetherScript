use super::support;

#[test]
fn package_directory_runs_and_checks() {
    let root = support::root("package-run");
    assert!(support::init(&root).status.success());

    let run = support::command(&["run", root.to_str().unwrap()], None);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "Hello from tetherscript!\n"
    );

    let check = support::command(&["check", root.to_str().unwrap()], None);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn nearest_package_runs_without_an_explicit_target() {
    let root = support::root("package-discovery");
    assert!(support::init(&root).status.success());
    let nested = root.join("src/nested");
    std::fs::create_dir_all(&nested).unwrap();

    let output = support::command(&["run"], Some(&nested));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Hello from tetherscript!\n"
    );
}
