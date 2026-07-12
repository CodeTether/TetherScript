use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Case {
    pub dir: PathBuf,
    pub source: PathBuf,
    pub next: PathBuf,
    pub exe: PathBuf,
}

pub fn temp_case() -> Case {
    let dir = std::env::temp_dir().join(format!(
        "tetherscript_reload_{}_{}",
        std::process::id(),
        now()
    ));
    Case {
        source: dir.join("agent.tether"),
        next: dir.join("next.tether"),
        exe: dir.join(format!("agent{}", std::env::consts::EXE_SUFFIX)),
        dir,
    }
}

pub fn build(source: &Path, exe: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["build"])
        .arg(source)
        .args(["-o"])
        .arg(exe)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
}

pub fn first_source(next: &Path) -> String {
    format!(
        "// tetherscript: hot-reload\nfn main() {{\n println(\"generation one\")\n fs.write(\"agent.tether\", fs.read(\"{}\").unwrap())\n fs.write(\".tetherscript/reload\", \"agent.tether\")\n}}\n",
        next.file_name().unwrap().to_string_lossy()
    )
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
