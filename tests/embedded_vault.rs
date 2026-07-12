#[allow(dead_code)]
mod embedded_reload_support;

use std::fs;
use std::process::Command;

use embedded_reload_support::{build, temp_case, text};

#[test]
fn vault_is_only_visible_to_authority_scripts() {
    assert_eq!(
        run("fn main() { println(global_defined(\"vault\")) }"),
        "false\n"
    );
    assert_eq!(
        run("// tetherscript: authority agent\nfn main() { println(global_defined(\"vault\")) }"),
        "true\n"
    );
}

fn run(source_text: &str) -> String {
    let case = temp_case();
    fs::create_dir_all(&case.dir).unwrap();
    fs::write(&case.source, source_text).unwrap();
    build(&case.source, &case.exe);
    let output = Command::new(&case.exe)
        .current_dir(&case.dir)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(case.dir);
    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    text(&output.stdout)
}
