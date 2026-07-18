use crate::browser_agent::{BrowserPage, RouteAction, RoutePattern, RouteRule};

#[test]
fn replay_reissues_latest_matching_request_with_body_patch() {
    let mut state = super::super::super::state::HostState::new();
    state.page = BrowserPage::from_html("https://app.test/", "<main></main>");
    state.page.route(
        RouteRule::new(RoutePattern::substring("/api")),
        RouteAction::fulfill(200, "replayed"),
    );
    state
        .page
        .eval_js("fetch('/api',{method:'POST',body:'old'});")
        .unwrap();
    let payload = super::super::super::value::map(vec![
        ("url_contains", super::super::super::value::string("/api")),
        ("body_patch", super::super::super::value::string("new")),
    ]);

    super::invoke(&mut state, &payload).unwrap();

    let log = state.page.route_log();
    assert_eq!(log.len(), 2);
    assert_eq!(log.last().unwrap().method, "POST");
    assert_eq!(log.last().unwrap().body.as_deref(), Some("new"));
}
