use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn selector_diagnostics_return_component_metadata() {
    let mut state = super::super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "https://app.test/",
        "<div id='root' data-component='App' data-react-state='ready' data-react-hooks='useState' data-react-owner='Root>App'></div>",
    );

    for kind in ["react.component_for_selector", "react.props"] {
        assert!(matches!(
            super::invoke(&mut state, kind, Some("#root")).unwrap(),
            Value::Map(_) | Value::List(_)
        ));
    }
    for (kind, expected) in [
        ("react.state", "ready"),
        ("react.hooks", "useState"),
        ("react.owner_stack", "Root>App"),
    ] {
        assert_eq!(
            super::invoke(&mut state, kind, Some("#root")).unwrap(),
            super::super::super::super::value::string(expected)
        );
    }
}
