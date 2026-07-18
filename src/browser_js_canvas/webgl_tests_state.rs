use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn scissor_and_color_mask_change_real_pixels_and_query_state() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='2'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         gl.clearColor(0,0,1,1);gl.clear(gl.COLOR_BUFFER_BIT);\
         gl.enable(gl.SCISSOR_TEST);gl.scissor(1,0,2,1);\
         gl.colorMask(true,false,false,true);gl.clearColor(1,1,0,1);\
         gl.clear(gl.COLOR_BUFFER_BIT);let bottom=new Uint8Array(4);\
         let top=new Uint8Array(4);gl.readPixels(1,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,bottom);\
         gl.readPixels(1,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,top);\
         let enabled=gl.isEnabled(gl.SCISSOR_TEST);let queried=gl.getParameter(gl.SCISSOR_TEST);\
         gl.disable(gl.SCISSOR_TEST);let disabled=!gl.isEnabled(gl.SCISSOR_TEST);\
         enabled+':'+queried+':'+disabled+':'+gl.getParameter(gl.SCISSOR_BOX).join(',')+\
         ':'+gl.getParameter(gl.COLOR_WRITEMASK).join(',')+'|'+bottom.join(',')+'|'+top.join(',');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String(
            "true:true:true:1,0,2,1:true,false,false,true|255,0,255,255|0,0,255,255".into()
        )
    );
}

#[test]
fn invalid_webgl_state_calls_set_and_clear_sticky_errors() {
    let result = eval_with_dom(
        "<canvas id='gl' width='1' height='1'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         gl.clearColor(0,1,0,1);gl.clear(gl.COLOR_BUFFER_BIT);gl.scissor(0,0,-1,1);\
         gl.enable(123);let a=gl.getError();let b=gl.getError();let c=gl.getError();\
         gl.clearColor(1,0,0,1);gl.clear(gl.COLOR_BUFFER_BIT|8);let d=gl.getError();\
         let unknown=gl.getParameter(999)===null;let e=gl.getError();let pixel=new Uint8Array(4);\
         gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);\
         [a,b,c,d,unknown,e,pixel.join(',')].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("1281|1280|0|1281|true|1280|0,255,0,255".into())
    );
}
