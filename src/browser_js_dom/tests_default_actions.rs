use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn label_click_activates_associated_control_unless_canceled() {
    assert_eq!(
        eval(
            "<label for='ok'>OK</label><input id='ok' type='checkbox'>",
            "let box=document.getElementById('ok');let label=document.querySelector('label');\
             let seen='';box.addEventListener('click',function(){seen=seen+'C';});\
             box.addEventListener('input',function(){seen=seen+'I';});\
             box.addEventListener('change',function(){seen=seen+'H';});\
             label.click();box.checked+':'+seen;",
        ),
        JsValue::String("true:CIH".into())
    );
    assert_eq!(
        eval(
            "<label for='ok'>OK</label><input id='ok' type='checkbox'>",
            "let label=document.querySelector('label');label.onclick=function(e){e.preventDefault();};\
             label.click();document.getElementById('ok').checked;",
        ),
        JsValue::Bool(false)
    );
}

#[test]
fn anchor_click_updates_location_without_agent_navigation() {
    assert_eq!(
        eval(
            "<a id='next' href='/next?ok=1#done'>Next</a>",
            "document.getElementById('next').click();location.href;",
        ),
        JsValue::String("http://localhost/next?ok=1#done".into())
    );
}

#[test]
fn form_submit_skips_event_while_request_submit_dispatches_it() {
    assert_eq!(
        eval(
            "<form id='f' action='/go'><input name='q' value='rust'></form>",
            "let f=document.getElementById('f');let seen='';\
             f.addEventListener('submit',function(){seen=seen+'S';});\
             let direct=f.submit();let requested=f.requestSubmit();\
             seen+':'+direct.action+':'+requested.action;",
        ),
        JsValue::String("S:/go:/go".into())
    );
}
