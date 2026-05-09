use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn screen_orientation_dispatches_change_listeners_and_handler() {
    let script = "let o=screen.orientation;let seen='';\
        function f(e){seen=seen+'L'+e.type+':' + (e.target===o) + ':' + (this===o)+';';}\
        o.addEventListener('change',f);\
        o.onchange=function(e){seen=seen+'H'+e.type+':' + (e.currentTarget===o);};\
        let ok=o.dispatchEvent({type:'change'});o.removeEventListener('change',f);\
        o.dispatchEvent({type:'change'});seen+':'+ok;";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(
        result.value,
        JsValue::String("Lchange:true:true;Hchange:trueHchange:true:true".into())
    );
}
