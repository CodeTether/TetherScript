use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator, NavigationKind};

const CASE: Case = Case {
    area: "html/browsers/history",
    wpt_shape: "same-document hash navigation appends traversable history",
    unsupported: &["full session history traversal algorithm"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html(
        "https://example.test/app",
        "<a id='hash' href='#details'>Details</a><main></main>",
    );
    let document = page.session.document.clone();
    page.click(&Locator::css("#hash")).unwrap();
    assert_eq!(page.session.url, "https://example.test/app#details");
    assert_eq!(page.session.document, document);
    assert_eq!(
        page.history_entries().last().unwrap().kind,
        NavigationKind::SameDocument
    );
    assert_eq!(page.go_back().kind, NavigationKind::SameDocument);
}
