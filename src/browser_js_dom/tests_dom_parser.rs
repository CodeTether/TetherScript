use super::*;

#[test]
fn dom_parser_normalizes_html_documents() {
    let result = eval_with_dom("<main></main>", "let d=DOMParser().parseFromString('<p id=\"x\">Hi</p>', 'text/html'); d.documentElement.nodeName + ':' + d.head.nodeName + ':' + d.body.firstChild.id + ':' + d.body.textContent;").unwrap();
    assert_eq!(result.value, JsValue::String("html:head:x:Hi".into()));
}
