use super::case::{assert_case, Case};
use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "html/semantics/scripting-1",
    wpt_shape: "module script fetches static import graph and evaluates dependencies first",
    unsupported: &["complete ESM namespace semantics", "import maps"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<script type='module' src='/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        module("import './leaf.js';window.order=window.order+'>app';"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/leaf.js")),
        module("window.order='leaf';"),
    );
    page.run_scripts().unwrap();
    assert_eq!(
        page.eval_js("window.order").unwrap(),
        JsValue::String("leaf>app".into())
    );
    assert!(page
        .route_log()
        .iter()
        .any(|entry| entry.url.ends_with("/leaf.js")));
}

fn module(source: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: Vec::new(),
        body: source.into(),
    })
}
