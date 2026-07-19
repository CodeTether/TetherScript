use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn indexed_draws_reject_invalid_types_ranges_and_deleted_bindings() {
    let result = eval_with_dom(
        "<canvas id='gl'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         let eb=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,eb);\
         gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint16Array([0,1,2]),gl.STATIC_DRAW);\
         gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_SHORT,1);let align=gl.getError();\
         gl.drawElements(gl.TRIANGLES,4,gl.UNSIGNED_SHORT,0);let range=gl.getError();\
         gl.drawElements(gl.TRIANGLES,3,gl.FLOAT,0);let kind=gl.getError();\
         gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_INT,0);let uint=gl.getError();\
         gl.deleteBuffer(eb);gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_SHORT,0);\
         [align,range,kind,uint,gl.getError(),gl.getParameter(gl.ELEMENT_ARRAY_BUFFER_BINDING)===null].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("1282|1282|1280|1280|1282|true".into())
    );
}
