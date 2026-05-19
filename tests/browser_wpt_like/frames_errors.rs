use super::case::{assert_case, Case};
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "html/browsers/windows/post-message",
    wpt_shape: "cross-origin frame message is blocked until policy allows origin",
    unsupported: &[
        "WindowProxy security wrappers",
        "origin-agent cluster isolation",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = r#"<iframe name="child" src="https://embed.test/widget"></iframe>"#;
    let mut page = BrowserPage::from_html("https://app.test/root", html);
    let tree = page.frame_tree();
    let child = tree.children_of(tree.root_id())[0].id();
    let blocked = page
        .post_frame_message(tree.root_id(), child, "blocked", "*")
        .unwrap();
    assert!(!blocked.same_origin);
    assert!(!blocked.allowed_by_policy);
    assert!(!blocked.delivered);
}
