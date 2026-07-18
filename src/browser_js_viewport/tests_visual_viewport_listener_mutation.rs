use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn listener_mutation_during_dispatch_matches_event_target_ordering() {
    let script = "let v=visualViewport;let count=0;let order='';\
        function again(){count=count+1;if(count==1){\
        v.addEventListener('resize',again,{once:true});}}\
        function later(){order=order+'L';}\
        function removeLater(){order=order+'K';\
        v.removeEventListener('resize',later);}\
        v.addEventListener('resize',again,{once:true});\
        v.addEventListener('resize',removeLater);\
        v.addEventListener('resize',later);resizeTo(100,30);\
        v.removeEventListener('resize',removeLater);\
        resizeTo(110,31);resizeTo(120,32);count+':'+order;";

    let result = eval_with_dom("", script).unwrap();
    assert_eq!(result.value, JsValue::String("2:K".into()));
}
