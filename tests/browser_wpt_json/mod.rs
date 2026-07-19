mod access;
mod model;
mod parse;
mod run;

#[test]
fn normalized_json_browser_fixtures_pass() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/browser_wpt_json/fixtures");
    let mut paths = std::fs::read_dir(root)
        .expect("read normalized browser fixtures")
        .map(|entry| entry.expect("read fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "at least one JSON browser fixture is required"
    );
    for path in paths {
        let source = std::fs::read_to_string(&path).expect("read browser fixture");
        let fixture =
            parse::fixture(&source).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        run::assert(&fixture);
    }
}
