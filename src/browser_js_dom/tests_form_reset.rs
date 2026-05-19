use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn button_type_reset_restores_text_input() {
    assert_eq!(
        eval(
            "<form><input id='t' name='q' value='hello'><button id='r' type='reset'></button></form>",
            "document.getElementById('t').value='world';\
             document.getElementById('r').click();\
             document.getElementById('t').value;",
        ),
        JsValue::String("hello".into())
    );
}

#[test]
fn input_type_reset_restores_checkbox() {
    assert_eq!(
        eval(
            "<form><input id='c' type='checkbox' checked name='ok'><input type='reset' id='r'></form>",
            "document.getElementById('c').checked=false;\
             document.getElementById('r').click();\
             document.getElementById('c').checked;",
        ),
        JsValue::Bool(true)
    );
}

#[test]
fn reset_event_is_cancelable() {
    assert_eq!(
        eval(
            "<form id='f'><input id='t' value='orig'><button id='r' type='reset'></button></form>",
            "document.getElementById('f').addEventListener('reset',function(e){e.preventDefault();});\
             document.getElementById('t').value='changed';\
             document.getElementById('r').click();\
             document.getElementById('t').value;",
        ),
        JsValue::String("changed".into())
    );
}

#[test]
fn reset_dispatches_event_before_restoring() {
    assert_eq!(
        eval(
            "<form id='f'><input id='t' value='init'><button id='r' type='reset'></button></form>",
            "let phase='';\
             document.getElementById('f').addEventListener('reset',function(){phase='saw:'+document.getElementById('t').value;});\
             document.getElementById('r').click();\
             phase;",
        ),
        JsValue::String("saw:init".into())
    );
}
