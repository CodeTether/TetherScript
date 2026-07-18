use crate::browser_agent::{BrowserPage, RouteAction, RoutePattern, RouteRule};
use crate::value::Value;

#[test]
fn request_uses_page_routes_and_returns_a_normalized_response() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("https://app.test/", "<main></main>");
    state.page.route(
        RouteRule::new(RoutePattern::substring("/api")),
        RouteAction::fulfill(202, "agent-body"),
    );
    let payload = super::super::super::value::map(vec![
        (
            "url",
            super::super::super::value::string("https://app.test/api"),
        ),
        ("method", super::super::super::value::string("post")),
        ("body", super::super::super::value::string("payload")),
    ]);

    let response = super::invoke(&mut state, "fetch", &payload).unwrap();

    assert_eq!(field(&response, "status"), Value::Int(202));
    assert_eq!(
        field(&response, "body"),
        super::super::super::value::string("agent-body")
    );
    assert_eq!(
        state.page.route_log().last().unwrap().body.as_deref(),
        Some("payload")
    );
}

fn field(value: &Value, name: &str) -> Value {
    let Value::Map(map) = value else {
        panic!("expected response map")
    };
    map.borrow().get(name).unwrap().clone()
}
