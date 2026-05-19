use super::case::{assert_case, Case};
use tetherscript::browser_agent::BrowserPage;

const CASE: Case = Case {
    area: "accname/html-aam/focus",
    wpt_shape: "accessibility names, states, hidden filtering, and focus order are visible",
    unsupported: &[
        "platform accessibility tree adapters",
        "complete ARIA role mapping",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<main><label for='email'>Email</label><input id='email'>\
        <img title='Logo'><button disabled aria-expanded='true'>Save</button>\
        <button aria-hidden='true'>Hide</button><a id='next' href='/x'>Next</a></main>";
    let page = BrowserPage::from_html("mem://a11y", html);
    let snapshot = page.accessibility_snapshot();
    let main = &snapshot.roots[0];
    assert_eq!(main.children[1].name, "Email");
    assert_eq!(main.children[2].name, "Logo");
    assert!(main.children.iter().any(|node| node.states.disabled));
    assert_eq!(snapshot.focus_order, vec!["#email", "#next"]);
    assert!(!format!("{snapshot:?}").contains("Hide"));
}
