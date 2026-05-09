use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn element_from_point_returns_live_dom_node() {
    let html = "<div id='a' style='width:10px;height:10px'>A</div>";
    let script = "let el=document.elementFromPoint(1,1);\
        el.id + ':' + el.getAttribute('id') + ':' + el.textContent;";
    let result = eval_with_dom(html, script).unwrap();

    assert_eq!(result.value, JsValue::String("a:a:A".into()));
}

#[test]
fn elements_from_point_orders_by_z_index_then_dom_order() {
    let html = "<div id='a' style='position:absolute;left:0;top:0;\
        width:10px;height:10px;z-index:1'></div>\
        <div id='b' style='position:absolute;left:0;top:0;\
        width:10px;height:10px;z-index:2'></div>";
    let script = "let xs=document.elementsFromPoint(2,2);\
        xs.length + ':' + xs[0].id + '>' + xs[1].id;";
    let result = eval_with_dom(html, script).unwrap();

    assert_eq!(result.value, JsValue::String("2:b>a".into()));
}

#[test]
fn element_from_point_returns_null_for_empty_point() {
    let html = "<main id='box' style='width:4px;height:4px'></main>";
    let result = eval_with_dom(html, "document.elementFromPoint(20,20) === null;").unwrap();

    assert_eq!(result.value, JsValue::Bool(true));
}
