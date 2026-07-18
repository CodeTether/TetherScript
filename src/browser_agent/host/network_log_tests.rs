use crate::browser_session::NetworkEvent;
use crate::value::Value;

#[test]
fn network_queries_filter_captured_events_and_failures() {
    let mut state = super::super::super::state::HostState::new();
    state.page.session.network = vec![
        NetworkEvent::new("GET", "https://app.test/ok", Some(200)),
        NetworkEvent::new("POST", "https://app.test/api", Some(503)),
    ];
    let payload = super::super::super::value::map(vec![(
        "url_contains",
        super::super::super::value::string("/api"),
    )]);

    let all = super::invoke(&state, &payload).unwrap();
    let failed_payload = super::super::super::value::map(vec![("failed_only", Value::Bool(true))]);
    let failed = super::invoke(&state, &failed_payload).unwrap();

    assert_eq!(length(all), 1);
    assert_eq!(length(failed), 1);
}

fn length(value: Value) -> usize {
    let Value::List(values) = value else {
        panic!("expected list")
    };
    values.borrow().len()
}
