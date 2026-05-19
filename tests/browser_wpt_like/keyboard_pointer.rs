use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "uievents",
    wpt_shape: "keyboard text insertion and pointer hover events mutate page state",
    unsupported: &["IME composition", "complete PointerEvent coordinate model"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<input id='q' value='a'><button id='go'>Go</button>\
        <script>window.hoverLog='';let b=document.getElementById('go');\
        b.addEventListener('mouseover',function(){window.hoverLog+='over>';});\
        b.addEventListener('mouseenter',function(){window.hoverLog+='enter>';});\
        b.addEventListener('mousemove',function(){window.hoverLog+='move';});</script>";
    let mut page = BrowserPage::from_html("mem://input", html);
    page.run_scripts().unwrap();
    page.press(&Locator::css("#q"), 'b').unwrap();
    page.hover(&Locator::css("#go")).unwrap();
    assert_eq!(
        page.eval_js("document.getElementById('q').value+':'+window.hoverLog")
            .unwrap(),
        JsValue::String("ab:over>enter>move".into())
    );
}
