use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn forms_collect_values_and_submit_events_can_cancel() {
    assert_eq!(
        eval(
            "<form id='f' action='/save' method='post'><input name='q' value='rust'>\
             <input type='checkbox' name='ok' checked><input type='checkbox' name='skip'>\
             <textarea name='body'>Hi</textarea></form>",
            "let form=document.getElementById('f');let data=form.collectFormData();\
             let submitted=form.requestSubmit();form.method+':'+submitted.action+':'\
             +submitted.method+':'+data.get('q')+':'+data.get('ok')+':'\
             +data.get('skip')+':'+submitted.data.get('body');",
        ),
        JsValue::String("post:/save:post:rust:on:null:Hi".into())
    );
    assert_eq!(
        eval(
            "<form id='f'><input name='q' value='rust'></form>",
            "let form=document.getElementById('f');let seen='';\
             form.addEventListener('submit',function(e){seen=e.type;e.preventDefault();});\
             let submitted=form.requestSubmit();seen+':'+submitted;",
        ),
        JsValue::String("submit:false".into())
    );
}

#[test]
fn request_submit_includes_submitter_data() {
    assert_eq!(
        eval(
            "<form id='f'><input name='q' value='rust'>\
             <button id='go' name='commit' value='yes'>Go</button></form>",
            "let f=document.getElementById('f');let go=document.getElementById('go');\
             f.requestSubmit(go).data.get('commit');",
        ),
        JsValue::String("yes".into())
    );
}
