use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/interaction/dnd",
    wpt_shape: "drag_to dispatches drag/drop events and pointer capture tracks owner",
    unsupported: &[
        "DataTransfer object construction",
        "real OS drag source negotiation",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<div id='src' draggable='true'>A</div><div id='dst'>B</div>\
        <script>window.log='';\
        document.getElementById('src').addEventListener('dragstart',function(){window.log+='ds>';});\
        document.getElementById('dst').addEventListener('drop',function(){window.log+='drop';});</script>";
    let mut page = BrowserPage::from_html("mem://drag", html);
    page.run_scripts().unwrap();
    page.drag_to(&Locator::css("#src"), &Locator::css("#dst"))
        .unwrap();
    assert_eq!(
        page.eval_js("window.log").unwrap(),
        tetherscript::js::JsValue::String("ds>drop".into())
    );
    page.set_pointer_capture(&Locator::css("#src")).unwrap();
    assert!(page.has_pointer_capture(&Locator::css("#src")).unwrap());
    assert!(page.release_pointer_capture(&Locator::css("#src")).unwrap());
    assert!(!page.release_pointer_capture(&Locator::css("#src")).unwrap());
}
