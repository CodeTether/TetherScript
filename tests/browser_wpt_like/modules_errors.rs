use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};

const CASE: Case = Case {
    area: "html/semantics/scripting-1",
    wpt_shape: "missing static module import reports the resolved chunk URL",
    unsupported: &["complete module graph error taxonomy"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<script type='module' src='/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/assets/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        module("import './missing.js';"),
    );
    let err = page.run_scripts().unwrap_err();
    assert!(
        err.contains("missing external script resource: ./missing.js"),
        "{err}"
    );
    assert!(err.contains("https://app.test/missing.js"), "{err}");
}

fn module(source: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: Vec::new(),
        body: source.into(),
    })
}
