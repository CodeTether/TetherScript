use crate::browser_agent::BrowserPage;
use crate::value::Value;

#[test]
fn console_and_react_diagnostics_use_live_page_state() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html(
        "https://app.test/",
        "<div id='root' data-reactroot></div><script>console.log('ready');console.error('React hydration mismatch');</script>",
    );
    state.page.run_scripts().unwrap();

    let logs = super::invoke(&mut state, &payload("console_logs", None)).unwrap();
    let errors = super::invoke(&mut state, &payload("console_errors", None)).unwrap();
    assert_eq!((length(logs), length(errors)), (2, 1));
    let detected = super::invoke(&mut state, &payload("react.detect", None)).unwrap();
    assert_eq!(detected, Value::Bool(true));
    let component = super::invoke(
        &mut state,
        &payload("react.component_for_selector", Some("#root")),
    )
    .unwrap();
    assert_eq!(
        field(&component, "tag"),
        super::super::super::value::string("div")
    );
}

fn payload(kind: &str, query: Option<&str>) -> Value {
    let mut entries = vec![("kind", super::super::super::value::string(kind))];
    if let Some(query) = query {
        entries.push(("query", super::super::super::value::string(query)));
    }
    super::super::super::value::map(entries)
}

fn length(value: Value) -> usize {
    let Value::List(values) = value else {
        panic!("expected list")
    };
    values.borrow().len()
}

fn field(value: &Value, name: &str) -> Value {
    let Value::Map(map) = value else {
        panic!("expected map")
    };
    map.borrow().get(name).unwrap().clone()
}
