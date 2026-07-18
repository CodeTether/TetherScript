use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn current_page_matches_its_native_png_baseline() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("https://app.test/", "<main>ready</main>");
    let evidence =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/browser-visual-unit");
    std::fs::create_dir_all(&evidence).unwrap();
    let path = evidence.join(format!(
        "tetherscript-visual-{}-{}.png",
        std::process::id(),
        state.page.screenshot().unwrap().pixels.len()
    ));
    std::fs::write(
        &path,
        super::super::super::png::encode(&state.page.screenshot().unwrap()),
    )
    .unwrap();
    let payload = super::super::super::value::map(vec![
        (
            "mode",
            super::super::super::value::string("assert_screenshot_matches"),
        ),
        (
            "before",
            super::super::super::value::string(path.to_string_lossy()),
        ),
    ]);

    let result = super::invoke(&state, &payload).unwrap();
    assert_eq!(field(&result, "matches"), Value::Bool(true));
}

fn field(value: &Value, name: &str) -> Value {
    let Value::Map(map) = value else {
        panic!("expected map")
    };
    map.borrow().get(name).unwrap().clone()
}
