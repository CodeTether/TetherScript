use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, RouteAction, RoutePattern, RouteRule};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "fetch/cors",
    wpt_shape: "blocked cross-origin response rejects with missing CORS header",
    unsupported: &["complete Fetch TypeError taxonomy"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/private")),
        RouteAction::fulfill(200, "private"),
    );
    page.eval_js(concat!(
        "window.out='';",
        "fetch('https://api.test/private')",
        ".catch(function(e){window.out='ERR:'+e;});"
    ))
    .unwrap();
    let JsValue::String(out) = page.eval_js("window.out").unwrap() else {
        panic!("expected CORS rejection string");
    };
    assert!(out.contains("missing access-control-allow-origin"));
}
