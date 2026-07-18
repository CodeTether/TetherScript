use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn scroll_metrics_styled_element_dimensions_and_position() {
    let result = eval_with_dom(
        "<main style='height:3px'></main><div id='box' style='width:12px;height:4px;border-width:2px'></div>",
        "let b=document.getElementById('box');\
         [b.clientWidth,b.clientHeight,b.scrollWidth,b.scrollHeight,\
          b.offsetLeft,b.offsetTop,b.clientLeft,b.clientTop].join(':');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("12:4:12:4:0:3:2:2".into()));
}

#[test]
fn scroll_metrics_zero_size_element_reports_zero_sizes() {
    let result = eval_with_dom(
        "<div id='box' style='width:0px;height:0px'></div>",
        "let b=document.getElementById('box');\
         [b.clientWidth,b.clientHeight,b.scrollWidth,b.scrollHeight].join(':');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("0:0:0:0".into()));
}

#[test]
fn element_scroll_methods_clamp_offsets_and_dispatch_non_bubbling_events() {
    let result = eval_with_dom(
        "<div id='outer'><div id='box' style='width:10px;height:4px;overflow:auto'>\
         <div id='content' style='width:25px;height:12px'></div></div></div>",
        "let outer=document.getElementById('outer');let b=document.getElementById('box');\
         let own=0;let parent=0;let handled=0;\
         outer.addEventListener('scroll',function(){parent=parent+1;});\
         b.addEventListener('scroll',function(e){if(!e.bubbles&&!e.cancelable){own=own+1;}});\
         b.onscroll=function(){handled=handled+1;};b.scrollTo(99,3);\
         b.scrollBy({left:-4,top:99});b.scrollBy(0,0);\
         let r=document.getElementById('content').getBoundingClientRect();\
         [b.scrollLeft,b.scrollTop,b.scrollWidth,b.scrollHeight,own,parent,handled,r.x,r.y].join(':');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("11:8:25:12:2:0:2:-11:-8".into())
    );
}
