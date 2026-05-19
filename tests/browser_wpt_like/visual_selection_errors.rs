use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "selection/html/rendering/screenshots",
    wpt_shape: "missing visual and selection targets return strict locator diagnostics",
    unsupported: &["fuzzy visual matching", "browser-native selection recovery"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("mem://errors", "<main></main>");
    let screenshot = page
        .element_screenshot(&Locator::css("#missing"))
        .unwrap_err();
    let selection = page.select_contents(&Locator::css("#missing")).unwrap_err();
    assert!(screenshot.contains("matched no elements"));
    assert!(selection.contains("matched no elements"));
}
