//! Trusted pointer sequence generation for coordinate clicks.

use crate::browser_agent::keyboard_escape::node;

pub(super) fn click(path: &[usize], x: i64, y: i64, page_x: i64, page_y: i64) -> String {
    let target = node(path);
    format!(
        "let n={target};\
         n.dispatchEvent({{type:'pointerdown',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'mousedown',isTrusted:true,{}}});\
         if(n.focus){{n.focus();}}\
         n.dispatchEvent({{type:'pointerup',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'mouseup',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'click',__agentClick:true,isTrusted:true,{}}})",
        pointer(1, x, y, page_x, page_y),
        mouse(1, 1, x, y, page_x, page_y),
        pointer(0, x, y, page_x, page_y),
        mouse(0, 1, x, y, page_x, page_y),
        mouse(0, 1, x, y, page_x, page_y)
    )
}

fn pointer(buttons: i64, x: i64, y: i64, page_x: i64, page_y: i64) -> String {
    let pressure = if buttons == 0 { "0" } else { "0.5" };
    format!(
        "{},pointerId:1,width:1,height:1,pressure:{pressure},tangentialPressure:0,\
         tiltX:0,tiltY:0,twist:0,pointerType:'mouse',isPrimary:true",
        mouse(buttons, 0, x, y, page_x, page_y)
    )
}

fn mouse(buttons: i64, detail: i64, x: i64, y: i64, page_x: i64, page_y: i64) -> String {
    format!(
        "button:0,buttons:{buttons},detail:{detail},clientX:{x},clientY:{y},screenX:{x},\
         screenY:{y},pageX:{page_x},pageY:{page_y},x:{x},y:{y},offsetX:0,offsetY:0,\
         movementX:0,movementY:0"
    )
}
