use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/browsing-the-web/history-and-session-state",
    wpt_shape: "page action traces and snapshot restore preserve observable state",
    unsupported: &[
        "back-forward cache",
        "complete session history serialization",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test", "<button id='go'>Go</button>");
    let before = page.capture_snapshot();
    page.click(&Locator::css("#go")).unwrap();
    let after = page.capture_snapshot();
    page.append_trace_entry("click#go", before, after);
    page.eval_js("localStorage.setItem('token','one');document.cookie='sid=abc';")
        .unwrap();
    let snapshot = page.snapshot_state();
    let mut restored = BrowserPage::new(Default::default());
    restored.restore_state(snapshot).unwrap();
    assert_eq!(page.trace_entries()[0].label, "click#go");
    assert_eq!(restored.session.local_storage_item("token"), Some("one"));
    assert!(restored
        .session
        .cookie_header("https://app.test")
        .contains("sid=abc"));
}
