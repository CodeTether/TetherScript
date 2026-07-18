use crate::browser_agent::BrowserPage;

#[test]
fn native_fill_replaces_value_and_tracks_focus() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("mem://fill", "<input id='entry' value='old'>");
    let payload = super::super::super::value::map(vec![
        ("selector", super::super::super::value::string("#entry")),
        ("value", super::super::super::value::string("native")),
    ]);
    super::super::invoke(&mut state, "fill_native", &payload).unwrap();
    assert_eq!(
        state
            .page
            .eval_js("document.querySelector('#entry').value")
            .unwrap()
            .display(),
        "native"
    );
    assert!(state.focused.is_some());
}
