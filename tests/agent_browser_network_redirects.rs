use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};
use tetherscript::js::JsValue;

fn redirect(status: u16, location: &str, cookie: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status,
        headers: vec![
            ("location".into(), location.into()),
            ("set-cookie".into(), cookie.into()),
        ],
        body: String::new(),
    })
}

fn ok(body: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: vec![("content-type".into(), "text/plain".into())],
        body: body.into(),
    })
}

#[test]
fn fetch_follows_302_rewrites_post_to_get_and_records_har_chain() {
    let mut page = BrowserPage::from_html("https://app.test/login", "<main></main>");
    page.route(
        RouteRule::new(RoutePattern::substring("/api/login")),
        redirect(302, "/dashboard", "sid=abc; Path=/; Secure; HttpOnly"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/dashboard")),
        ok("dashboard-ok"),
    );

    page.eval_js("window.out=''; fetch('/api/login',{method:'post',headers:{'content-type':'text/plain'},body:'payload'}).then(function(r){ r.text().then(function(t){ window.out=r.url+'|'+r.method+'|'+t; }); });").unwrap();

    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("https://app.test/dashboard|GET|dashboard-ok".into())
    );
    let logs = page.route_log();
    let final_request = logs
        .iter()
        .find(|entry| entry.url.ends_with("/dashboard"))
        .unwrap();
    assert_eq!(final_request.method, "GET");
    assert_eq!(final_request.body, None);
    assert!(final_request
        .headers
        .contains(&("cookie".into(), "sid=abc".into())));
    let har = page.production_debug_report().network_har;
    let redirect_hop = har
        .iter()
        .find(|entry| entry.request.url.ends_with("/api/login"))
        .unwrap();
    let final_hop = har
        .iter()
        .find(|entry| entry.request.url.ends_with("/dashboard"))
        .unwrap();
    assert_eq!(redirect_hop.response.status, 302);
    assert_eq!(final_hop.request.url, "https://app.test/dashboard");
    assert_eq!(final_hop.response.status, 200);
}

#[test]
fn xhr_follows_307_preserves_post_body_and_response_url() {
    let mut page = BrowserPage::from_html("https://app.test/form", "<main></main>");
    page.route(
        RouteRule::new(RoutePattern::substring("/api/submit")),
        redirect(307, "/api/final", "sid=xyz; Path=/; Secure"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/api/final")),
        ok("ok"),
    );

    page.eval_js("window.out=''; let x=XMLHttpRequest(); x.onload=function(){ window.out=x.responseURL+'|'+x.status+'|'+x.responseText; }; x.open('post','/api/submit'); x.send('payload');").unwrap();

    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("https://app.test/api/final|200|ok".into())
    );
    let logs = page.route_log();
    let final_request = logs
        .iter()
        .find(|entry| entry.url.ends_with("/api/final"))
        .unwrap();
    assert_eq!(final_request.method, "POST");
    assert_eq!(final_request.body.as_deref(), Some("payload"));
    assert!(final_request
        .headers
        .contains(&("cookie".into(), "sid=xyz".into())));
}
