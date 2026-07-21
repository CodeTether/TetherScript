use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(args)
        .output()
        .expect("tetherscript should run")
}

#[test]
fn namespaced_modules_run_in_both_engines() {
    for engine in ["--vm", "--interp"] {
        let output = run(&["run", engine, "examples/modules.tether"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "sum 42\ndouble 42\n"
        );
    }
}

#[test]
fn missing_export_is_path_qualified() {
    let root = unique_dir();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.tether"), "import \"./lib.tether\" as lib\n").unwrap();
    std::fs::write(root.join("lib.tether"), "export missing\n").unwrap();
    let output = run(&["check", root.join("main.tether").to_str().unwrap()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("export `missing` is not declared"),
        "{stderr}"
    );
}

fn unique_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tetherscript-modules-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
