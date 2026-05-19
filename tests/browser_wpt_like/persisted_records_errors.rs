use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserContext, CacheResponse};

const CASE: Case = Case {
    area: "storage/cache-storage/indexeddb",
    wpt_shape: "missing persisted records stay origin scoped and return empty results",
    unsupported: &["quota eviction", "storage partitioning by top-level site"],
};

pub fn run() {
    assert_case(&CASE);
    let mut context = BrowserContext::new();
    context.cache_put(
        "https://other.test",
        "v1",
        "/api/data",
        CacheResponse::text(200, "cached"),
    );
    assert!(context
        .cache_match("https://app.test", "v1", "/api/data")
        .is_none());
    assert_eq!(
        context.indexed_db_get("https://app.test", "app", "users", "missing"),
        None
    );
}
