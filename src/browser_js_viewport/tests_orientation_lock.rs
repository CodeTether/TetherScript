use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn orientation_lock_updates_state_and_returns_real_promise() {
    let script = "let o=screen.orientation;let seen='';\
        o.addEventListener('change',function(e){seen=e.type+':'+e.isTrusted+':'\
        +o.type+':'+o.angle;});let p=o.lock('portrait-primary');\
        [typeof o.lock,p.__promise_state,typeof p.finally,p.__promise_value===undefined,\
        o.type,o.angle,seen].join('|');";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(
        result.value,
        JsValue::String(
            "function|fulfilled|function|true|portrait-primary|90|change:true:portrait-primary:90"
                .into()
        )
    );
}

#[test]
fn fixed_lock_survives_resize_and_unlock_restores_viewport_orientation() {
    let script = "resizeTo(20,100);let o=screen.orientation;let seen='';\
        o.addEventListener('change',function(){seen=seen+o.type+':'+o.angle+';';});\
        let p=o.lock('landscape-secondary');let locked=o.type+':'+o.angle;\
        resizeTo(10,200);let during=o.type+':'+o.angle;let ret=o.unlock();\
        [p.__promise_state,locked,during,ret===undefined,o.type+':'+o.angle,seen].join('|');";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(result.value, JsValue::String(
        "fulfilled|landscape-secondary:180|landscape-secondary:180|true|portrait-primary:90|landscape-secondary:180;portrait-primary:90;".into()
    ));
}

#[test]
fn invalid_orientation_lock_rejects_without_mutating_state() {
    let script = "let o=screen.orientation;let before=o.type+':'+o.angle;\
        let p=o.lock('sideways');let after=o.type+':'+o.angle;\
        [p.__promise_state,p.__promise_reason,before===after].join('|');";
    let result = eval_with_dom("", script).unwrap();

    assert_eq!(result.value, JsValue::String(
        "rejected|NotSupportedError: screen.orientation.lock: invalid orientation 'sideways'|true".into()
    ));
}
