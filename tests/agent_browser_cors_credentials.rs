use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};
use tetherscript::browser_cookie::{Cookie, SameSite};
use tetherscript::js::JsValue;

fn api_cookie() -> Cookie {
    Cookie {
        name: "sid".into(),
        value: "abc".into(),
        domain: "api.test".into(),
        path: "/".into(),
        secure: true,
        http_only: true,
        same_site: SameSite::None,
        expires_at: None,
        host_only: false,
    }
}

fn preflight() -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 204,
        headers: vec![
            (
                "access-control-allow-origin".into(),
                "https://app.test".into(),
            ),
            ("access-control-allow-credentials".into(), "true".into()),
            ("access-control-allow-methods".into(), "POST".into()),
            (
                "access-control-allow-headers".into(),
                "content-type, x-csrf".into(),
            ),
        ],
        body: String::new(),
    })
}

fn cors_text(body: &str, credentials: bool) -> RouteAction {
    let mut headers = vec![(
        "access-control-allow-origin".into(),
        "https://app.test".into(),
    )];
    if credentials {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
    }
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers,
        body: body.into(),
    })
}

#[test]
fn fetch_include_credentials_preflights_and_sends_cookie() {
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.session.cookies.push(api_cookie());
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/data")).method("OPTIONS"),
        preflight(),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/data")).method("POST"),
        cors_text("ok", true),
    );

    page.eval_js("window.out=''; fetch('https://api.test/data',{method:'post',credentials:'include',headers:{'x-csrf':'1','content-type':'application/json'},body:'{}'}).then(function(r){ r.text().then(function(t){ window.out=t; }); }).catch(function(e){ window.out='ERR:'+e; });").unwrap();

    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("ok".into())
    );
    let logs = page.route_log();
    let options = logs.iter().find(|entry| entry.method == "OPTIONS").unwrap();
    let requested = header(&options.headers, "access-control-request-headers").unwrap();
    assert_eq!(header(&options.headers, "origin"), Some("https://app.test"));
    assert_eq!(
        header(&options.headers, "access-control-request-method"),
        Some("POST")
    );
    assert!(requested.contains("content-type"));
    assert!(requested.contains("x-csrf"));
    let post = logs.iter().find(|entry| entry.method == "POST").unwrap();
    assert_eq!(header(&post.headers, "origin"), Some("https://app.test"));
    assert_eq!(header(&post.headers, "cookie"), Some("sid=abc"));
}

#[test]
fn fetch_default_credentials_skip_cross_origin_cookie() {
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.session.cookies.push(api_cookie());
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/public")),
        RouteAction::Fulfill(RouteFulfillment {
            status: 200,
            headers: vec![("access-control-allow-origin".into(), "*".into())],
            body: "public".into(),
        }),
    );

    page.eval_js("window.out=''; fetch('https://api.test/public').then(function(r){ r.text().then(function(t){ window.out=t; }); });").unwrap();

    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("public".into())
    );
    let log = page.route_log().pop().unwrap();
    assert_eq!(header(&log.headers, "origin"), Some("https://app.test"));
    assert_eq!(header(&log.headers, "cookie"), None);
}

#[test]
fn missing_cors_response_header_rejects_fetch() {
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/private")),
        RouteAction::fulfill(200, "private"),
    );

    page.eval_js("window.out=''; fetch('https://api.test/private').catch(function(e){ window.out='ERR:'+e; });").unwrap();

    let JsValue::String(out) = page.eval_js("window.out").unwrap() else {
        panic!("expected string output");
    };
    assert!(out.contains("missing access-control-allow-origin"));
    assert_eq!(
        page.session.network.last().unwrap().route_result.as_deref(),
        Some("fulfill")
    );
}

#[test]
fn xhr_with_credentials_sends_cross_origin_cookie() {
    let mut page = BrowserPage::from_html("https://app.test/", "<main></main>");
    page.allow_origin("https://api.test");
    page.session.cookies.push(api_cookie());
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/profile")),
        cors_text("profile", true),
    );

    page.eval_js("window.out=''; let x=XMLHttpRequest(); x.withCredentials=true; x.onload=function(){ window.out=x.status+':'+x.responseText; }; x.onerror=function(){ window.out='ERR:'+x.statusText; }; x.open('GET','https://api.test/profile'); x.send();").unwrap();

    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("200:profile".into())
    );
    let log = page.route_log().pop().unwrap();
    assert_eq!(header(&log.headers, "origin"), Some("https://app.test"));
    assert_eq!(header(&log.headers, "cookie"), Some("sid=abc"));
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
