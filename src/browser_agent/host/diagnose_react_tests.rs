use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn every_react_diagnostic_has_a_native_result() {
    let mut state = super::super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "https://app.test/",
        "<div id='root' data-reactroot></div><div id='suspense'></div><script>window.React={version:'18.3'};console.error('React hydration mismatch');</script>",
    );
    state.page.run_scripts().unwrap();

    for kind in [
        "react.component_tree",
        "react.errors",
        "react.hydration_warnings",
        "react.suspense_boundaries",
    ] {
        assert!(matches!(
            super::invoke(&mut state, kind, None).unwrap(),
            Value::List(_)
        ));
    }
    assert_eq!(
        super::invoke(&mut state, "react.detect", None).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        super::invoke(&mut state, "react.version", None).unwrap(),
        string("18.3")
    );
}

fn string(value: &str) -> Value {
    super::super::super::super::value::string(value)
}
