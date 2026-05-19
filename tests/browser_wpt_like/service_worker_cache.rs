use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserContext, BrowserPage, CacheResponse};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "service-workers/cache-storage",
    wpt_shape: "active service worker fulfills pass-through fetch from CacheStorage",
    unsupported: &[
        "real worker thread execution",
        "complete fetch event lifecycle",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let mut context = BrowserContext::new();
    let index = context.new_page(BrowserPage::from_html("https://app.test/", ""));
    let page = context.page_mut(index).unwrap();
    page.service_worker_register("/", "/sw.js").unwrap();
    page.service_worker_activate("/").unwrap();
    page.cache_put("v1", "/api/data", CacheResponse::text(200, "cached"))
        .unwrap();
    page.eval_js(
        "window.out='';fetch('/api/data').then(function(r){\
        r.text().then(function(t){window.out=r.status+':'+t;});});",
    )
    .unwrap();
    assert_eq!(
        page.eval_js("window.out").unwrap(),
        JsValue::String("200:cached".into())
    );
    let log = page.service_worker_fetch_log().unwrap();
    assert!(log[0].matched);
    assert_eq!(log[0].cache_name.as_deref(), Some("v1"));
}
