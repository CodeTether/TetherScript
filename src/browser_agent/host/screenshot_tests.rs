use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn selector_screenshot_is_cropped_before_encoding() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "https://app.test/",
        "<main id='box' style='width:3px;height:2px;background:green'></main>",
    );
    let full = super::invoke(&state, &super::super::value::map(Vec::new())).unwrap();
    let selector =
        super::super::value::map(vec![("selector", super::super::value::string("#box"))]);
    let cropped = super::invoke(&state, &selector).unwrap();

    assert!(byte_len(cropped) < byte_len(full));
}

fn byte_len(value: Value) -> usize {
    let Value::Bytes(bytes) = value else {
        panic!("expected screenshot bytes")
    };
    bytes.borrow().len()
}
