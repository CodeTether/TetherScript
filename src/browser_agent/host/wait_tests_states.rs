use crate::browser_agent::BrowserPage;

#[test]
fn selector_wait_supports_each_document_state() {
    let mut state = super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "mem://states",
        "<div id='hidden' style='display:none'></div>",
    );
    for (selector, desired) in [
        ("#hidden", "attached"),
        ("#hidden", "hidden"),
        ("#missing", "detached"),
    ] {
        let payload = super::payload(vec![
            ("selector", super::super::value::string(selector)),
            ("state", super::super::value::string(desired)),
            ("timeout_ms", crate::value::Value::Int(0)),
        ]);
        assert!(super::super::wait::invoke(&mut state, &payload)
            .unwrap()
            .truthy());
    }
    let invalid = super::payload(vec![
        ("selector", super::super::value::string("#hidden")),
        ("state", super::super::value::string("painted")),
    ]);
    assert!(super::super::wait::invoke(&mut state, &invalid)
        .unwrap_err()
        .contains("unsupported selector state `painted`"));
}
