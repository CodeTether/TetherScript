use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, RoutePattern, RouteRule};
use tetherscript::js::JsValue;

mod route;

const CASE: Case = Case {
    area: "fetch/cors",
    wpt_shape: "cross-origin POST with custom header preflights and validates CORS",
    unsupported: &["full fetch error taxonomy", "streaming bodies"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/data")).method("OPTIONS"),
        route::preflight(),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/data")).method("POST"),
        route::cors_text("ok"),
    );
    page.eval_js(concat!(
        "window.out='';",
        "fetch('https://api.test/data',{method:'POST',headers:{'x-test':'1'},body:'q'})",
        ".then(function(r){r.text().then(function(t){window.out=t;});})",
        ".catch(function(e){window.out='ERR:'+e;});"
    ))
    .unwrap();
    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("ok".into())
    );
    let logs = page.route_log();
    assert!(logs.iter().any(|entry| entry.method == "OPTIONS"));
    assert!(logs.iter().any(|entry| entry.method == "POST"));
}
