use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

#[test]
fn navigator_connection_is_deterministic() {
    let result = eval_with_dom(
        "<main></main>",
        "let c=navigator.connection;\
         [c.effectiveType,c.type,c.downlink,c.downlinkMax,c.rtt,c.saveData,c.onchange].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("4g|wifi|10|10|50|false|".into())
    );
}
