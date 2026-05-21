//! Touch-event JavaScript generation.

use crate::browser_agent::action::BoundingBox;
use crate::browser_agent::interact::pointer_event_fields;

/// Build a touch event dispatch script.
pub fn touch_script(path: &[usize], bounds: BoundingBox, include_move: bool) -> String {
    let move_event = if include_move {
        format!(
            "n.dispatchEvent({{type:'touchmove',touches:t.touches,targetTouches:t.targetTouches,changedTouches:[{{{}}}]}});",
            pointer_event_fields::touch(bounds)
        )
    } else {
        String::new()
    };
    format!(
        "let n={};let t={{type:'touchstart',touches:[{{{}}}],\
         targetTouches:[{{{}}}],changedTouches:[{{{}}}]}};\
         n.dispatchEvent(t);{}n.dispatchEvent({{type:'touchend',touches:[],\
         targetTouches:[],changedTouches:t.changedTouches}});",
        crate::browser_agent::keyboard_escape::node(path),
        pointer_event_fields::touch(bounds),
        pointer_event_fields::touch(bounds),
        pointer_event_fields::touch(bounds),
        move_event
    )
}
