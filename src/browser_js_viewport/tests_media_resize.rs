use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn resize_updates_live_media_query_and_dispatches_change() {
    let script = "let m=matchMedia('(min-width:100px)');let seen='';\
        m.addEventListener('change',function(e){seen=e.matches+':'+e.media+':'+e.isTrusted;});\
        resizeTo(120,40);[m.matches,seen].join('|');";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(
        result.value,
        JsValue::String("true|true:(min-width:100px):true".into())
    );
}
