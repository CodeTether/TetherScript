use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/interaction/focus",
    wpt_shape: "focus order skips disabled controls and Tab advances active element",
    unsupported: &[
        "shadow-root focus delegation",
        "platform-specific tab stops",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<input id='one'><button id='skip' disabled>Skip</button>\
        <input id='two'><input id='three' disabled>";
    let mut page = BrowserPage::from_html("mem://focus", html);
    let selectors = page
        .focus_order()
        .into_iter()
        .map(|target| target.selector)
        .collect::<Vec<_>>();
    assert_eq!(selectors, vec!["#one", "#two"]);
    page.press(&Locator::css("#one"), "Tab").unwrap();
    assert_eq!(page.session.focus.as_deref(), Some("#two"));
}
