use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(super) fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tetherscript-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

pub(super) fn command(args: &[&str], cwd: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tetherscript"));
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.output().expect("tetherscript should run")
}

pub(super) fn init(root: &Path) -> Output {
    command(&["init", root.to_str().unwrap()], None)
}
