use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn canvas_resize_preserves_webgl_resources_and_updates_buffer_dimensions() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let canvas=document.getElementById('gl');let gl=canvas.getContext('webgl');\
         let buffer=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buffer);\
         gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([1,2]),gl.STATIC_DRAW);\
         canvas.width=8;canvas.height=6;gl.bindBuffer(gl.ARRAY_BUFFER,buffer);\
         [gl.isBuffer(buffer),gl.getParameter(gl.ARRAY_BUFFER_BINDING)===buffer,\
         gl.getParameter(gl.MAX_VIEWPORT_DIMS).join(','),\
         gl.getBufferParameter(gl.ARRAY_BUFFER,gl.BUFFER_SIZE),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("true|true|8,6|8|0".into()));
}
