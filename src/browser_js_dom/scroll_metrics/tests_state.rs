use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn options_properties_and_shifted_handles_share_live_scroll_state() {
    let result = eval_with_dom(
        "<main id='root'><div id='box' style='width:10px;height:4px;overflow:auto'>\
         <div style='width:25px;height:12px'></div></div></main>",
        "let root=document.getElementById('root');let b=document.getElementById('box');\
         b.scrollTo({top:5});b.scrollLeft=7;b.scrollTop=99;\
         let marker=document.createElement('i');root.prepend(marker);\
         let fresh=document.getElementById('box');\
         [b.scrollLeft,b.scrollTop,fresh.scrollLeft,fresh.scrollTop].join(':');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("7:8:7:8".into()));
}

#[test]
fn element_scroll_state_resets_between_browser_runtimes() {
    let html =
        "<div id='box' style='height:2px;overflow:auto'><div style='height:5px'></div></div>";
    eval_with_dom(html, "let b=document.getElementById('box');b.scrollTop=3;").unwrap();
    let result = eval_with_dom(html, "document.getElementById('box').scrollTop;").unwrap();

    assert_eq!(result.value, JsValue::Number(0.0));
}
