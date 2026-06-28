use std::process::Command;

#[test]
fn run_passes_script_args_to_env_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/cli_args.tether", "one", "two words"])
        .output()
        .expect("run tetherscript");

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "argc 2\narg one\narg two words\n");
}

#[test]
fn run_accepts_dash_args_after_separator() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/cli_args.tether", "--", "--name", "Riley"])
        .output()
        .expect("run tetherscript");

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "argc 2\narg --name\narg Riley\n");
}

#[test]
fn build_creates_standalone_launcher_with_args() {
    let exe = env!("CARGO_BIN_EXE_tetherscript");
    let out = std::env::temp_dir().join(format!(
        "tetherscript_cli_args_{}{}",
        std::process::id(),
        std::env::consts::EXE_SUFFIX
    ));
    let build = Command::new(exe)
        .args(["build", "examples/cli_args.tether", "-o"])
        .arg(&out)
        .output()
        .expect("build launcher");

    assert!(build.status.success(), "stderr: {}", text(&build.stderr));
    let run = Command::new(&out).args(["one", "two"]).output().unwrap();
    assert!(run.status.success(), "stderr: {}", text(&run.stderr));
    assert_eq!(text(&run.stdout), "argc 2\narg one\narg two\n");
    let _ = std::fs::remove_file(out);
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
