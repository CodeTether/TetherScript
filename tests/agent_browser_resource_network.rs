use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};
use tetherscript::browser_cookie::{Cookie, SameSite};

fn text(status: u16, body: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status,
        headers: Vec::new(),
        body: body.into(),
    })
}

fn with_headers(status: u16, headers: Vec<(&str, &str)>, body: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status,
        headers: headers
            .into_iter()
            .map(|(name, value)| (name.into(), value.into()))
            .collect(),
        body: body.into(),
    })
}

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

#[test]
fn module_script_redirect_chain_records_har_and_refreshes_cookie() {
    let html = "<main id='out'></main><script type='module' src='/assets/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.session
        .set_cookie("seed=old; Path=/; Secure; HttpOnly")
        .unwrap();
    page.route(
        RouteRule::new(RoutePattern::substring("/assets/app.js")),
        with_headers(
            302,
            vec![
                ("location", "/assets/app.v1.js"),
                ("set-cookie", "sid=abc; Path=/; Secure; HttpOnly"),
            ],
            "",
        ),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/assets/app.v1.js")),
        text(
            200,
            "document.getElementById('out').textContent = 'resource routed';",
        ),
    );

    page.run_scripts().unwrap();

    assert!(page.session.html.contains(">resource routed<"));
    let final_log = page
        .route_log()
        .into_iter()
        .find(|entry| entry.url.ends_with("/assets/app.v1.js"))
        .unwrap();
    let cookie = header(&final_log.headers, "cookie").unwrap();
    assert!(cookie.contains("seed=old"));
    assert!(cookie.contains("sid=abc"));
    let har = page.production_debug_report().network_har;
    assert_eq!(status_for(&har, "/assets/app.js"), Some(302));
    assert_eq!(status_for(&har, "/assets/app.v1.js"), Some(200));
}

#[test]
fn cross_origin_module_script_sends_origin_but_not_cookie_by_default() {
    let html =
        "<main id='out'></main><script type='module' src='https://api.test/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.allow_origin("https://api.test");
    page.session.cookies.push(api_cookie());
    page.route(
        RouteRule::new(RoutePattern::substring("api.test/app.js")),
        with_headers(
            200,
            vec![("access-control-allow-origin", "https://app.test")],
            "document.getElementById('out').textContent = 'api module';",
        ),
    );

    page.run_scripts().unwrap();

    assert!(page.session.html.contains(">api module<"));
    let log = page.route_log().pop().unwrap();
    assert_eq!(header(&log.headers, "origin"), Some("https://app.test"));
    assert_eq!(header(&log.headers, "cookie"), None);
}

#[test]
fn stylesheet_image_and_source_map_resources_are_har_visible() {
    let html = concat!(
        "<link rel='stylesheet' href='/app.css'>",
        "<img src='/logo.png'>",
        "<script src='/app.js'></script>",
    );
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.css")),
        text(200, "#box { color: green; }"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/logo.png")),
        text(200, "png-bytes"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js.map")),
        text(200, "{}"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        text(200, "console.log('app');\n//# sourceMappingURL=app.js.map"),
    );

    page.run_scripts().unwrap();

    assert!(page.session.css.contains("color: green"));
    assert_eq!(
        page.image_resource_metadata()[0].byte_len,
        "png-bytes".len()
    );
    let report = page.production_debug_report();
    assert!(report.source_maps[0].registered);
    assert_eq!(status_for(&report.network_har, "/app.css"), Some(200));
    assert_eq!(status_for(&report.network_har, "/logo.png"), Some(200));
    assert_eq!(status_for(&report.network_har, "/app.js.map"), Some(200));
}

#[test]
fn missing_subresource_validation_names_element_and_resolved_url() {
    let html = "<img src='logo.png'>";
    let page = BrowserPage::from_html("https://app.test/assets/index.html", html);

    let err = page.validate_external_resources().unwrap_err();

    assert!(err.contains("image logo.png"));
    assert!(err.contains("https://app.test/assets/logo.png"));
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn status_for(har: &[tetherscript::browser_agent::BrowserHarEntry], suffix: &str) -> Option<u16> {
    har.iter()
        .find(|entry| entry.request.url.ends_with(suffix))
        .map(|entry| entry.response.status)
}
