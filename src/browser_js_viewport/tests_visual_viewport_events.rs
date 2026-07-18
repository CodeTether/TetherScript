use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn manual_dispatch_normalizes_and_cancels_events() {
    let script = "let v=window.visualViewport;let seen='';\
        let event={type:'resize',cancelable:true};\
        v.onresize=function(e){seen=[e.type,e.target===v,this===v,\
        e.isTrusted,e.bubbles,e.cancelable].join(':');e.preventDefault();};\
        let ok=v.dispatchEvent(event);seen+':'+ok+':'+event.defaultPrevented;";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(
        result.value,
        JsValue::String("resize:true:true:false:false:true:false:true".into())
    );
}

#[test]
fn browser_resize_and_scroll_dispatch_live_viewport_events() {
    let script = "let v=visualViewport;let out='';\
        function keep(e){out=out+e.type+':'+e.isTrusted+':' +(e.target===v)+':'\
        +(this===v)+':'+v.width+'x'+v.height+';';}\
        function once(){out=out+'once;';}\
        v.addEventListener('resize',keep);v.addEventListener('resize',keep);\
        v.addEventListener('resize',once,{once:true});\
        resizeTo(120,40);resizeTo(140,50);\
        v.removeEventListener('resize',keep);resizeTo(160,60);\
        v.addEventListener('scroll',function(e){out=out+'scroll:'+v.pageLeft+','\
        +v.pageTop+':'+e.isTrusted+';';});\
        v.onscrollend=function(e){out=out+'scrollend:'+e.isTrusted;};scrollTo(3,5);\
        out+'|'+[v.width,v.height,v.pageLeft,v.pageTop].join(':');";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(result.value, JsValue::String(
        "resize:true:true:true:120x40;once;resize:true:true:true:140x50;scroll:3,5:true;scrollend:true|160:60:3:5".into()
    ));
}
