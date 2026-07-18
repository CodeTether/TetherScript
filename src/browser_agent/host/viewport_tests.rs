use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn viewport_action_updates_page_and_javascript_media() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://viewport-host", "<main></main>");
    let payload =
        super::super::value::map(vec![("width", Value::Int(120)), ("height", Value::Int(40))]);

    let result = super::invoke(&mut state, &payload).unwrap();
    let js = state
        .page
        .eval_js("innerWidth+':'+innerHeight+':'+matchMedia('(min-width:100px)').matches")
        .unwrap();

    assert_eq!(state.page.viewport_width, 120);
    assert_eq!(state.page.viewport_height, 40);
    assert_eq!(js.display(), "120:40:true");
    assert!(matches!(result, Value::Map(_)));
}
