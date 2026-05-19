use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserContext, BrowserPage};

const CASE: Case = Case {
    area: "IndexedDB",
    wpt_shape: "same-origin pages share records and separate contexts do not",
    unsupported: &["transaction scheduling", "structured clone value storage"],
};

pub fn run() {
    assert_case(&CASE);
    let mut context = BrowserContext::new();
    let first = context.new_page(BrowserPage::from_html("https://app.test/a", ""));
    let second = context.new_page(BrowserPage::from_html("https://app.test/b", ""));
    context
        .page_mut(first)
        .unwrap()
        .indexed_db_put("app", "users", "1", "Ada")
        .unwrap();
    assert_eq!(
        context
            .page_mut(second)
            .unwrap()
            .indexed_db_get("app", "users", "1")
            .unwrap(),
        Some("Ada".into())
    );
    assert_eq!(
        BrowserContext::new().indexed_db_get("https://app.test", "app", "users", "1"),
        None
    );
}
