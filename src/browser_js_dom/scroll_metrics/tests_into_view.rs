use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn scroll_into_view_reveals_ancestors_and_updates_hit_testing() {
    let result = eval_with_dom(
        "<div id='box' style='width:10px;height:4px;overflow:auto'>\
         <div style='height:8px'></div><button id='target' style='height:2px'>T</button></div>",
        "let b=document.getElementById('box');let t=document.getElementById('target');\
         let seen=0;b.addEventListener('scroll',function(){seen=seen+1;});\
         t.scrollIntoView({block:'nearest'});\
         [b.scrollTop,window.scrollY,seen,document.elementFromPoint(1,2).id].join(':');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("6:0:1:target".into()));
}

#[test]
fn scroll_into_view_aligns_nested_scroll_containers() {
    let result = eval_with_dom(
        "<div id='box' style='width:10px;height:4px;overflow:auto'>\
         <div style='height:8px'></div><button id='target' style='height:2px'>T</button>\
         <div style='height:2px'></div></div>",
        "let b=document.getElementById('box');let t=document.getElementById('target');\
         t.scrollIntoView({block:'center'});let r=t.getBoundingClientRect();\
         [b.scrollTop,r.y,document.elementFromPoint(1,1).id].join(':');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("7:1:target".into()));
}

#[test]
fn scroll_into_view_honors_viewport_alignment_and_dispatches_window_scroll() {
    let result = eval_with_dom(
        "<div style='height:30px'></div><div id='target' style='height:4px'></div>",
        "let seen=0;window.addEventListener('scroll',function(){seen=seen+1;});\
         let t=document.getElementById('target');t.scrollIntoView({block:'center'});\
         let r=t.getBoundingClientRect();\
         [window.scrollX,window.scrollY,window.pageYOffset,pageYOffset,seen,r.y,\
          document.elementFromPoint(1,10).id].join(':');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("0:20:20:20:1:10:target".into())
    );
}
