mod extract;
mod harness;

use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

const DOCUMENT_FRAGMENT_QUERY: &str =
    include_str!("fixtures/dom/nodes/DocumentFragment-querySelectorAll-after-modification.html");

#[test]
fn upstream_document_fragment_query_survives_mutation() {
    let scripts = extract::inline_scripts(DOCUMENT_FRAGMENT_QUERY);
    assert_eq!(scripts.len(), 1, "expected one upstream inline test script");
    let script = format!(
        "{}\n{}\n__wpt_failures.join('|');",
        harness::SOURCE,
        scripts.join("\n")
    );
    let result = eval_with_dom(DOCUMENT_FRAGMENT_QUERY, &script).unwrap();
    assert_eq!(result.value, JsValue::String(String::new()));
}
