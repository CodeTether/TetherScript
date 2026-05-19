use super::case::{assert_case, Case};
use tetherscript::browser_agent::diff::visual::compare_rasters;
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/rendering/screenshots",
    wpt_shape: "screenshots and visual diff reflect deterministic DOM mutation",
    unsupported: &[
        "font rasterization parity",
        "anti-aliasing and subpixel paint model",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<main id='box' style='background:red;width:4px;height:4px'></main>";
    let mut page = BrowserPage::from_html("mem://visual", html);
    page.set_viewport_size(4, 4).unwrap();
    let before = page.screenshot().unwrap();
    page.eval_js(
        "document.querySelector('#box').setAttribute('style',\
        'background:blue;width:4px;height:4px');",
    )
    .unwrap();
    let after = page.screenshot().unwrap();
    let diff = compare_rasters(&before, &after);
    let element = page.element_screenshot(&Locator::css("#box")).unwrap();
    assert_eq!((before.width, before.height), (4, 4));
    assert!(diff.stats.changed_pixels > 0);
    assert_eq!((element.image.width, element.image.height), (4, 4));
}
