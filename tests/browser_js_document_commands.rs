use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn command_queries_track_the_active_editing_host() {
    let value = eval(
        "<input id='q' value='alpha'>",
        "let q=document.getElementById('q');let d=q.ownerDocument;q.focus();\
         let before=d.queryCommandEnabled('copy');\
         let selected=d.execCommand('selectAll');\
         let after=d.queryCommandEnabled('COPY');\
         let inserted=d.execCommand('insertText',false,'omega');\
         d.queryCommandSupported('insertText')+':'+d.queryCommandSupported('bold')+':'\
         +before+':'+selected+':'+after+':'+inserted+':'\
         +document.getElementById('q').value;",
    );

    assert_eq!(
        value,
        JsValue::String("true:false:false:true:true:true:omega".into())
    );
}

#[test]
fn copy_cut_and_paste_mutate_focused_controls() {
    let value = eval(
        "<input id='a' value='abcd'><input id='b' value='zz'>",
        "let a=document.getElementById('a');let b=document.getElementById('b');\
         let events=[];a.addEventListener('copy',function(e){\
         events.push(e.type+':'+e.clipboardData.types[0]);});\
         b.addEventListener('paste',function(e){\
         events.push(e.type+':'+e.clipboardData.getData('text/plain'));});\
         b.addEventListener('cut',function(e){\
         events.push(e.type+':'+e.clipboardData.types[0]);});\
         a.focus();a.setSelectionRange(1,3);let copied=document.execCommand('copy');\
         b.focus();b.setSelectionRange(1,1);let pasted=document.execCommand('paste');\
         b.setSelectionRange(1,3);let cut=document.execCommand('cut');\
         a.focus();a.setSelectionRange(4,4);document.execCommand('paste');\
         copied+':'+pasted+':'+cut+':'+document.getElementById('a').value+':'\
         +document.getElementById('b').value+':'+events.join(',');",
    );

    assert_eq!(
        value,
        JsValue::String("true:true:true:abcdbc:zz:copy:text/plain,paste:bc,cut:text/plain".into())
    );
}

#[test]
fn clipboard_handlers_can_override_or_cancel_default_edits() {
    let value = eval(
        "<input id='a' value='x'><input id='b' value='y'><input id='c' value='z'>",
        "let a=document.getElementById('a');let b=document.getElementById('b');\
         let c=document.getElementById('c');a.addEventListener('copy',function(e){\
         e.clipboardData.setData('text/plain','custom');e.preventDefault();});\
         b.addEventListener('paste',function(e){e.preventDefault();});\
         a.focus();a.select();document.execCommand('copy');\
         b.focus();b.select();document.execCommand('paste');\
         c.focus();c.select();document.execCommand('paste');\
         document.getElementById('b').value+':'+document.getElementById('c').value;",
    );

    assert_eq!(value, JsValue::String("y:custom".into()));
}

#[test]
fn select_all_and_delete_edit_contenteditable_text() {
    let value = eval(
        "<div id='editor' contenteditable>hello</div>",
        "let e=document.getElementById('editor');e.focus();\
         let selected=document.execCommand('selectAll');\
         let removed=document.execCommand('delete');\
         let inserted=document.execCommand('insertText',false,'ready');\
         selected+':'+removed+':'+inserted+':'\
         +document.getElementById('editor').textContent;",
    );

    assert_eq!(value, JsValue::String("true:true:true:ready".into()));
}

#[test]
fn clipboard_state_resets_between_browser_runtimes() {
    let copied = eval(
        "<input id='q' value='secret'>",
        "let q=document.getElementById('q');q.focus();q.select();\
         document.execCommand('copy');",
    );
    let enabled = eval(
        "<input id='q'>",
        "let q=document.getElementById('q');q.focus();\
         document.queryCommandEnabled('paste');",
    );

    assert_eq!(copied, JsValue::Bool(true));
    assert_eq!(enabled, JsValue::Bool(false));
}
