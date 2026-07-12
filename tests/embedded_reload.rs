mod embedded_reload_support;

use std::fs;
use std::process::Command;

use embedded_reload_support::{build, first_source, temp_case, text};

#[test]
fn embedded_launcher_reloads_sidecar_source() {
    let case = temp_case();
    fs::create_dir_all(case.dir.join(".tetherscript")).unwrap();
    fs::write(&case.source, first_source(&case.next)).unwrap();
    fs::write(&case.next, "fn main() { println(\"generation two\") }\n").unwrap();
    build(&case.source, &case.exe);

    let output = Command::new(&case.exe)
        .current_dir(&case.dir)
        .arg("--reload-source")
        .arg(&case.source)
        .arg("--grant-fs")
        .arg(&case.dir)
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "generation one\ngeneration two\n");
    let _ = fs::remove_dir_all(case.dir);
}
