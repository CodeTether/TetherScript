use std::process::Command;

#[test]
fn import_cycles_report_the_full_chain() {
    let root = unique_dir();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.tether"), "import \"./b.tether\" as b\n").unwrap();
    std::fs::write(root.join("b.tether"), "import \"./a.tether\" as a\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["check", root.join("a.tether").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("module import cycle"), "{stderr}");
    assert!(
        stderr.contains("a.tether") && stderr.contains("b.tether"),
        "{stderr}"
    );
}

#[test]
fn import_cannot_escape_a_package() {
    let outer = unique_dir();
    let root = outer.join("package");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("tetherscript.json"), "{}").unwrap();
    std::fs::write(
        outer.join("outside.tether"),
        "export value\nlet value = 1\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/main.tether"),
        "import \"../../outside.tether\" as out\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["check", root.join("src/main.tether").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("escapes package root"));
}

fn unique_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tetherscript-module-errors-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
