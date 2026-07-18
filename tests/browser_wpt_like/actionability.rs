use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/interaction/actionability",
    wpt_shape: "visible, enabled, attached, and stable hit-target checks gate actions",
    unsupported: &["continuous CSS/Web Animations stability sampling"],
};

pub fn run() {
    assert_case(&CASE);
    visible_enabled_hidden_and_disabled();
    waits_for_stable_bounds_before_action();
    removed_during_wait_is_strict_locator_error();
    overlapping_z_index_resolves_before_click();
}

fn visible_enabled_hidden_and_disabled() {
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

fn waits_for_stable_bounds_before_action() {
    let html = "<button id='later' style='position:absolute;left:0px'>Later</button>\
        <script style='display:none'>let b=document.getElementById('later');\
        b.addEventListener('click',function(){window.clicked='yes';});\
        setTimeout(function(){b.setAttribute('style','position:absolute;left:12px');},5);</script>";
    let mut page = BrowserPage::from_html("mem://actionability-visible", html);

    let report = page.click(&Locator::css("#later")).unwrap();

    assert_eq!(report.bounds.x, 12);
    assert_eq!(page.eval_js("window.clicked").unwrap().display(), "yes");
}

fn removed_during_wait_is_strict_locator_error() {
    let html = "<button id='gone' style='display:none'>Gone</button>\
        <script>setTimeout(function(){document.getElementById('gone').remove();},5);</script>";
    let mut page = BrowserPage::from_html("mem://actionability-removed", html);
    page.run_scripts().unwrap();

    let err = page.click(&Locator::css("#gone")).unwrap_err();

    assert!(
        err.contains("matched no elements")
            || err.contains("strict locator")
            || err.contains("expected exactly one"),
        "removed element should surface a strict locator error, got: {err}"
    );
}

fn overlapping_z_index_resolves_before_click() {
    let html = "<button id='target' style='width:20px;height:8px;position:absolute;left:0;top:0;z-index:1'>T</button>\
        <div id='cover' style='width:20px;height:8px;position:absolute;left:0;top:0;z-index:2'></div>\
        <script>window.clicked='no';document.getElementById('target').addEventListener('click',function(){window.clicked='yes';});\
        setTimeout(function(){document.getElementById('cover').setAttribute('style','display:none');},5);</script>";
    let mut page = BrowserPage::from_html("mem://actionability-overlap", html);
    page.run_scripts().unwrap();

    page.click(&Locator::css("#target")).unwrap();

    assert_eq!(page.eval_js("window.clicked").unwrap().display(), "yes");
}
