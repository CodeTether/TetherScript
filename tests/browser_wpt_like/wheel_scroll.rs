use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "uievents/wheel",
    wpt_shape: "wheel and element scroll offsets move visible content",
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
    element_scroll_moves_geometry_and_hit_testing();
}

fn element_scroll_moves_geometry_and_hit_testing() {
    let html = "<div id='box' style='width:10px;height:4px;overflow:auto'>\
        <div style='height:8px'></div><button id='target' style='width:25px;height:2px'>T</button></div>";
    let mut page = BrowserPage::from_html("mem://element-scroll", html);
    let value = page
        .eval_js(
            "let b=document.getElementById('box');b.scrollTo(99,99);\
             let r=document.getElementById('target').getBoundingClientRect();\
             [b.scrollLeft,b.scrollTop,r.x,r.y,document.elementFromPoint(1,2).id].join(':');",
        )
        .unwrap();

    assert_eq!(value.display(), "15:6:-15:2:target");
}
