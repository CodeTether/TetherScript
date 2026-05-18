use tetherscript::browser_agent::{
    BrowserPage, RouteAction, RouteFulfillment, RoutePattern, RouteRule,
};

fn module(source: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: Vec::new(),
        body: source.into(),
    })
}

#[test]
fn module_script_executes_static_import_graph() {
    let html = concat!(
        "<main id='out'></main>",
        "<script type='module' src='/assets/app.js'></script>",
    );
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);

    page.register_script_resource(
        "/assets/app.js",
        "import { boot as start } from './chunk.js'; start();",
    );
    page.register_script_resource(
        "https://app.test/assets/chunk.js",
        "export function boot(){ document.getElementById('out').textContent = 'booted'; }",
    );
    page.run_scripts().unwrap();

    assert!(page.session.html.contains(">booted<"));
}

#[test]
fn missing_static_import_reports_resolved_chunk() {
    let html = "<script type='module' src='/assets/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);

    page.register_script_resource("/assets/app.js", "import './missing.js';");
    let err = page.run_scripts().unwrap_err();

    assert!(err.contains("missing external script resource: ./missing.js"));
    assert!(err.contains("https://app.test/assets/missing.js"));
}

#[test]
fn resource_validation_checks_static_module_imports() {
    let html = "<script type='module' src='/assets/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);

    page.register_script_resource("/assets/app.js", "import './missing.js';");
    let err = page.validate_external_resources().unwrap_err();

    assert!(err.contains("missing external script resource: ./missing.js"));
    assert!(err.contains("https://app.test/assets/missing.js"));
}

#[test]
fn static_imports_fetch_through_route_network() {
    let html = "<main id='out'></main><script type='module' src='/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        module("import { boot } from './chunk.js'; boot();"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/chunk.js")),
        module("export function boot(){ document.getElementById('out').textContent='net'; }"),
    );

    page.run_scripts().unwrap();

    assert!(page.session.html.contains(">net<"));
    assert!(page
        .route_log()
        .iter()
        .any(|entry| entry.url.ends_with("/chunk.js")));
}

#[test]
fn modulepreload_fetch_is_deduped_with_static_import() {
    let html = concat!(
        "<link rel='modulepreload' href='/chunk.js'>",
        "<script type='module' src='/app.js'></script>",
    );
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        module("import './chunk.js';"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/chunk.js")),
        module("window.loaded = true;"),
    );

    page.run_scripts().unwrap();

    let chunk_requests = page
        .route_log()
        .iter()
        .filter(|entry| entry.url.ends_with("/chunk.js"))
        .count();
    assert_eq!(chunk_requests, 1);
}

#[test]
fn nested_static_imports_evaluate_dependencies_first() {
    let html = "<script type='module' src='/app.js'></script>";
    let mut page = BrowserPage::from_html("https://app.test/index.html", html);
    page.route(
        RouteRule::new(RoutePattern::substring("/app.js")),
        module("import './mid.js'; window.order = window.order + '>app';"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/mid.js")),
        module("import './leaf.js'; window.order = window.order + '>mid';"),
    );
    page.route(
        RouteRule::new(RoutePattern::substring("/leaf.js")),
        module("window.order = 'leaf';"),
    );

    page.run_scripts().unwrap();

    assert_eq!(
        page.eval_js("window.order").unwrap(),
        tetherscript::js::JsValue::String("leaf>mid>app".into())
    );
}
