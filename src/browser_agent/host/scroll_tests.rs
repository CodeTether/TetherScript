use crate::browser_agent::BrowserPage;
use crate::browser_session::ScrollState;

#[test]
fn selector_and_coordinate_scroll_update_page_and_js_state() {
    let mut state = super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "mem://scroll",
        "<div style='height:1200px'></div><button id='below'>below</button>",
    );
    let selector = super::value::map(vec![("selector", super::value::string("#below"))]);
    super::scroll::invoke(&mut state, &selector).unwrap();
    assert!(state.page.session.scroll.y > 0);
    let in_view_y = state.page.session.scroll.y;
    assert_eq!(js_scroll_y(&mut state), state.page.session.scroll.y);

    let coordinates = super::value::map(vec![
        ("x", crate::value::Value::Int(7)),
        ("y", crate::value::Value::Int(11)),
    ]);
    super::scroll::invoke(&mut state, &coordinates).unwrap();
    assert_eq!(state.page.session.scroll, ScrollState { x: 7, y: 11 });
    assert_eq!(js_scroll_y(&mut state), 11);

    let combined = super::value::map(vec![
        ("selector", super::value::string("#below")),
        ("x", crate::value::Value::Int(5)),
        ("y", crate::value::Value::Int(6)),
    ]);
    super::scroll::invoke(&mut state, &combined).unwrap();
    assert_eq!(state.page.session.scroll.y, in_view_y + 6);
}

fn js_scroll_y(state: &mut super::state::HostState) -> i64 {
    match state.page.eval_js("window.scrollY").unwrap() {
        crate::js::JsValue::Number(value) => value as i64,
        value => panic!("expected integer scrollY, got {value:?}"),
    }
}
