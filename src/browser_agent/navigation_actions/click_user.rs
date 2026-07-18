//! User-like click script generation.

use crate::browser_agent::action::BoundingBox;
use crate::browser_agent::interact::pointer_event_fields as fields;
use crate::browser_agent::keyboard_escape::node;

pub(crate) fn script(path: &[usize], bounds: BoundingBox) -> String {
    let target = node(path);
    format!(
        "let n={target};\
         n.dispatchEvent({{type:'pointerdown',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'mousedown',isTrusted:true,{}}});\
         if(n.focus){{n.focus();}}\
         n.dispatchEvent({{type:'pointerup',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'mouseup',isTrusted:true,{}}});\
         n.dispatchEvent({{type:'click',__agentClick:true,isTrusted:true,{}}})",
        fields::pointer(1, bounds),
        fields::mouse(1, 1, bounds),
        fields::pointer(0, bounds),
        fields::mouse(0, 1, bounds),
        fields::mouse(0, 1, bounds)
    )
}
