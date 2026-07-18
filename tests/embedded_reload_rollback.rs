#[allow(dead_code)]
mod embedded_reload_support;

use std::fs;
use std::process::Command;

use embedded_reload_support::{build, temp_case, text};

#[test]
fn embedded_launcher_restores_runtime_failed_generation() {
    let case = temp_case();
    fs::create_dir_all(case.dir.join(".tetherscript")).unwrap();
    let first = first_source();
    let failed = "fn main() { println(\"generation two\") let broken = 1 / 0 println(broken) }\n";
    fs::write(&case.source, &first).unwrap();
    fs::write(&case.next, failed).unwrap();
    build(&case.source, &case.exe);

    let output = Command::new(&case.exe)
        .current_dir(&case.dir)
        .args(["--reload-source", "agent.tether"])
        .args(["--grant-fs", case.dir.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert_eq!(fs::read_to_string(&case.source).unwrap(), first);
    assert_eq!(
        fs::read_to_string(case.source.with_extension("tether.failed")).unwrap(),
        failed
    );
    assert!(text(&output.stdout).contains("generation one recovered"));
    assert!(text(&output.stderr).contains("restored"));
    let _ = fs::remove_dir_all(case.dir);
}

fn first_source() -> String {
    "// tetherscript: hot-reload\nfn main() {\n if fs.exists(\"attempted.txt\").unwrap() {\n  println(\"generation one recovered\")\n } else {\n  fs.write(\"attempted.txt\", \"yes\").unwrap()\n  println(\"generation one\")\n  fs.write(\"agent.tether\", fs.read(\"next.tether\").unwrap()).unwrap()\n  fs.write(\".tetherscript/reload\", \"agent.tether\").unwrap()\n }\n}\n".into()
}
