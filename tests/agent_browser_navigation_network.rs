use tetherscript::browser_agent::{
    BrowserHarEntry, BrowserPage, Locator, PageEventKind, RouteAction, RouteFulfillment,
    RoutePattern, RouteRule,
};

fn redirect(status: u16, location: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status,
        headers: vec![("location".into(), location.into())],
        body: String::new(),
    })
}

fn redirect_cookie(status: u16, location: &str, cookie: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status,
        headers: vec![
            ("location".into(), location.into()),
            ("set-cookie".into(), cookie.into()),
        ],
        body: String::new(),
    })
}

fn html(body: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: vec![("content-type".into(), "text/html".into())],
        body: body.into(),
    })
}

#[test]
fn location_href_follows_redirect_and_commits_final_url() {
    let mut page = BrowserPage::from_html("https://app.test/start", "<main></main>");
    page.route(
        RouteRule::new(RoutePattern::substring("/hop")),
        redirect(302, "/final"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/final")),
        html("<main id='done'>Done</main>"),
    );

    page.eval_js("location.href='/hop';").unwrap();

    assert_eq!(page.session.url, "https://app.test/final");
    assert!(page.session.html.contains("id='done'"));
    assert_eq!(page.route_log()[0].url, "https://app.test/hop");
    assert_eq!(
        status_for(&page.production_debug_report().network_har, "/final"),
        Some(200)
    );
}

#[test]
fn history_push_state_does_not_fetch_document_route() {
    let mut page = BrowserPage::from_html("https://app.test/start", "<main></main>");
    page.route(
        RouteRule::new(RoutePattern::substring("/client")),
        html("<main>Network</main>"),
    );

    page.eval_js("history.pushState({page:1}, '', '/client');")
        .unwrap();

    assert!(page.route_log().is_empty());
    assert!(!page.session.html.contains("Network"));
}

#[test]
fn anchor_click_records_lifecycle_and_har_navigation_entry() {
    let mut page = BrowserPage::from_html(
        "https://app.test/start",
        "<a id='next' href='/next'>Next</a>",
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/next")),
        html("<main>Next</main>"),
    );
    let start = page.event_log().len();

    page.click(&Locator::css("#next")).unwrap();

    let nav: Vec<_> = page.event_log()[start..]
        .iter()
        .filter(|event| event.kind == PageEventKind::Navigation)
        .map(|event| event.action.as_str())
        .collect();
    assert_eq!(nav, ["beforeunload", "unload", "document_replace"]);
    assert_eq!(
        status_for(&page.production_debug_report().network_har, "/next"),
        Some(200)
    );
}

#[test]
fn form_post_preserves_body_cookies_redirect_and_final_url() {
    let markup = "<form action='/submit' method='post'>\
        <input name='email' value='a@b.test'>\
        <button id='go' name='intent' value='login'>Go</button>\
    </form>";
    let mut page = BrowserPage::from_html("https://app.test/form", markup);
    page.session.set_cookie("sid=old; Path=/; Secure").unwrap();
    page.route(
        RouteRule::new(RoutePattern::substring("/submit")).method("POST"),
        redirect_cookie(307, "/final", "sid=new; Path=/; Secure"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/final")).method("POST"),
        html("<main>Final</main>"),
    );

    page.click(&Locator::css("#go")).unwrap();

    let logs = page.route_log();
    let submit = logs
        .iter()
        .find(|entry| entry.url.ends_with("/submit"))
        .unwrap();
    let final_hop = logs
        .iter()
        .find(|entry| entry.url.ends_with("/final"))
        .unwrap();
    assert_eq!(page.session.url, "https://app.test/final");
    assert_eq!(
        submit.body.as_deref(),
        Some("email=a%40b.test&intent=login")
    );
    assert!(submit
        .headers
        .contains(&("cookie".into(), "sid=old".into())));
    assert_eq!(final_hop.method, "POST");
    assert_eq!(final_hop.body, submit.body);
    assert!(final_hop
        .headers
        .contains(&("cookie".into(), "sid=new".into())));
}

#[test]
fn back_forward_preserve_history_after_redirected_navigation() {
    let mut page = BrowserPage::from_html(
        "https://app.test/start",
        "<a id='go' href='/redirect'>Go</a>",
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/redirect")),
        redirect(302, "/final"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/final")),
        html("<a id='third' href='/third'>Third</a>"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/third")),
        html("<main>Third</main>"),
    );

    page.click(&Locator::css("#go")).unwrap();
    page.click(&Locator::css("#third")).unwrap();
    assert_eq!(page.go_back().navigation.url, "https://app.test/final");
    assert_eq!(page.go_forward().navigation.url, "https://app.test/third");

    let urls: Vec<_> = page
        .history_entries()
        .into_iter()
        .map(|entry| entry.url)
        .collect();
    assert!(urls.contains(&"https://app.test/final".into()));
    assert!(!urls.contains(&"https://app.test/redirect".into()));
}

fn status_for(har: &[BrowserHarEntry], suffix: &str) -> Option<u16> {
    har.iter()
        .find(|entry| entry.request.url.ends_with(suffix))
        .map(|entry| entry.response.status)
}
