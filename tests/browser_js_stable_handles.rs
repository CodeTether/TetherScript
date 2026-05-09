use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

fn eval(script: &str) -> JsValue {
    eval_with_dom(
        "<main><button id='a'></button><button id='b'></button></main>",
        script,
    )
    .unwrap()
    .value
}

#[test]
fn shifted_element_handle_methods_use_current_node() {
    let value = eval(
        "let a=document.getElementById('a');let b=document.getElementById('b');\
         a.remove();b.setAttribute('data-live','1');\
         b.appendChild(document.createTextNode('ok'));\
         b.addEventListener('click',function(){this.setAttribute('data-hit','yes');});\
         b.click();let fresh=document.getElementById('b');\
         fresh.getAttribute('data-live')+':'\
         +fresh.getAttribute('data-hit')+':'+fresh.textContent;",
    );

    assert_eq!(value, JsValue::String("1:yes:ok".into()));
}
