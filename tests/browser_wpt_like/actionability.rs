use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/interaction/actionability",
    wpt_shape: "visible and enabled elements pass actionability while hidden and disabled fail",
    unsupported: &[
        "stable visibility animation gate",
        "attached-to-document check",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<button id='ok'>Go</button>\
        <button id='hid' style='display:none'>Hide</button>\
        <button id='dis' disabled>Off</button>";
    let mut page = BrowserPage::from_html("mem://actionability", html);
    let ok = page.click(&Locator::css("#ok"));
    let hid = page.click(&Locator::css("#hid"));
    let dis = page.click(&Locator::css("#dis"));
    assert!(
        ok.is_ok(),
        "visible enabled button should pass actionability"
    );
    assert!(hid.is_err(), "hidden button should fail actionability");
    assert!(dis.is_err(), "disabled button should fail actionability");
}
