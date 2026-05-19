use super::case::{assert_case, Case};
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "html/browsers/windows/post-message",
    wpt_shape: "iframe postMessage delivery follows target origin metadata",
    unsupported: &[
        "script-visible WindowProxy objects",
        "nested event-loop dispatch timing",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = r#"<iframe name="child" src="https://app.test/child"></iframe>"#;
    let mut page = BrowserPage::from_html("https://app.test/root", html);
    let tree = page.frame_tree();
    let root = tree.root_id();
    let child = tree.children_of(root)[0].id();
    let delivered = page
        .post_frame_message(root, child, "ready", "https://app.test")
        .unwrap();
    let mismatched = page
        .post_frame_message(root, child, "blocked", "https://other.test")
        .unwrap();
    assert!(delivered.same_origin);
    assert!(delivered.delivered);
    assert_eq!(page.frame_messages_for(child)[0].data, "ready");
    assert!(!mismatched.delivered);
}
