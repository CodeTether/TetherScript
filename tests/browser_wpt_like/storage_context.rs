use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserContext, BrowserPage};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "storage/context",
    wpt_shape: "same-context pages share cookies/localStorage but not sessionStorage",
    unsupported: &["quota", "storage partitioning by top-level site"],
};

pub fn run() {
    assert_case(&CASE);
    let mut context = BrowserContext::new();
    let first = context.new_page(BrowserPage::from_html("https://example.test/a", ""));
    let second = context.new_page(BrowserPage::from_html("https://example.test/b", ""));
    context.page_mut(first).unwrap().eval_js(seed()).unwrap();
    let value = context.page_mut(second).unwrap().eval_js(read()).unwrap();
    assert_eq!(value, JsValue::String("one:sid=abc:null".into()));
}

fn seed() -> &'static str {
    "localStorage.setItem('token','one');\
     sessionStorage.setItem('tab','left');document.cookie='sid=abc';"
}

fn read() -> &'static str {
    "localStorage.getItem('token')+':'+document.cookie+':'\
     +sessionStorage.getItem('tab')"
}
