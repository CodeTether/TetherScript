use crate::browser_agent::BrowserPage;
use crate::value::Value;

use super::super::super::super::state::HostState;

#[test]
fn wait_settles_a_delayed_fetch() {
    let mut state = HostState::new();
    state.page = BrowserPage::from_html(
        "https://app.test",
        "<script>setTimeout(function(){fetch('/later');},0);</script>",
    );
    let payload = payload("/later", "request", 50);

    let Value::List(events) = super::super::invoke(&mut state, &payload).unwrap() else {
        panic!("expected network event list")
    };

    assert_eq!(events.borrow().len(), 1);
}

#[test]
fn wait_reports_timeout_for_missing_event() {
    let mut state = HostState::new();
    let error = super::super::invoke(&mut state, &payload("/never", "request", 1)).unwrap_err();

    assert!(error.contains("network request containing `/never` timed out"));
}

fn payload(url: &str, kind: &str, timeout: i64) -> Value {
    super::super::super::super::value::map(vec![
        (
            "url_contains",
            super::super::super::super::value::string(url),
        ),
        ("wait_kind", super::super::super::super::value::string(kind)),
        ("timeout_ms", Value::Int(timeout)),
    ])
}
