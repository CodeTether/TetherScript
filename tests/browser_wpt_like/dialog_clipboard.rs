use super::case::{assert_case, Case};
use tetherscript::browser_agent::permissions::BrowserPermission;
use tetherscript::browser_agent::{BrowserPage, Locator};
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "html/webappapis/user-prompts/clipboard-apis",
    wpt_shape: "queued prompt decisions and clipboard text APIs update page state",
    unsupported: &[
        "modal event-loop blocking",
        "system clipboard and user activation gating",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<button id='copy'>Copy <span>me</span></button><input id='target'>";
    let mut page = BrowserPage::from_html("https://app.test", html);
    page.grant_permission("https://app.test", BrowserPermission::ClipboardRead);
    page.grant_permission("https://app.test", BrowserPermission::ClipboardWrite);
    page.accept_next_dialog();
    let confirmed = page.eval_js("window.confirm('continue?')").unwrap();
    page.accept_next_prompt("Grace");
    let prompted = page.eval_js("window.prompt('Name?', 'Ada')").unwrap();
    page.copy_text(&Locator::css("#copy")).unwrap();
    page.paste(&Locator::css("#target")).unwrap();
    page.eval_js("navigator.clipboard.writeText('browser clip');")
        .unwrap();
    let text = "let out='';navigator.clipboard.readText().then(function(v){out=v;});out;";
    assert_eq!(confirmed, JsValue::Bool(true));
    assert_eq!(prompted, JsValue::String("Grace".into()));
    assert_eq!(
        page.eval_js("document.getElementById('target').value")
            .unwrap(),
        JsValue::String("Copy me".into())
    );
    assert_eq!(
        page.eval_js(text).unwrap(),
        JsValue::String("browser clip".into())
    );
    assert_eq!(page.dialogs().len(), 2);
}
