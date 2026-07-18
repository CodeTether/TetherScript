use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "selection",
    wpt_shape: "selection ranges and legacy editing commands mutate focused hosts",
    unsupported: &[
        "multi-range selection",
        "bidirectional text selection geometry",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html(
        "https://example.test",
        "<main><p id='note'>Alpha <b>Beta</b></p></main>",
    );
    page.select_contents(&Locator::css("#note")).unwrap();
    assert_eq!(page.selection_text().unwrap(), "Alpha Beta");
    let mut input = BrowserPage::from_html(
        "https://example.test",
        "<input id='q' value='search'><script>\
        let q=document.getElementById('q');q.focus();q.setSelectionRange(1,4);\
        </script>",
    );
    input.run_scripts().unwrap();
    assert_eq!(input.selection_text().unwrap(), "ear");
    let edited = input
        .eval_js(
            "document.execCommand('copy');let q=document.getElementById('q');\
             q.setSelectionRange(6,6);document.execCommand('paste');\
             document.getElementById('q').value;",
        )
        .unwrap();
    assert_eq!(
        edited,
        tetherscript::js::JsValue::String("searchear".into())
    );
}
