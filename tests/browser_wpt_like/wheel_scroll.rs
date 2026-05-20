use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "uievents/wheel",
    wpt_shape: "wheel events update viewport scroll offset with delta application",
    unsupported: &["smooth scrolling", "scrollbar hit testing"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<div id='box'><p id='tall'>line</p></div>";
    let mut page = BrowserPage::from_html("mem://wheel", html);
    page.wheel(&Locator::css("#tall"), 0, 50).unwrap();
    assert_eq!(
        page.session.scroll.y, 50,
        "wheel delta Y should update scroll"
    );
    page.wheel(&Locator::css("#tall"), 10, 0).unwrap();
    assert_eq!(
        page.session.scroll.x, 10,
        "wheel delta X should update scroll"
    );
}
