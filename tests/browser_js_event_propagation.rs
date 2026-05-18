use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

fn eval(script: &str) -> JsValue {
    eval_with_dom("<div id='p'><button id='c'>Go</button></div>", script)
        .unwrap()
        .value
}

#[test]
fn stop_propagation_preserves_same_capture_target_listeners() {
    let value = eval(
        "let p=document.getElementById('p');let c=document.getElementById('c');\
         let seen='';\
         p.addEventListener('click',function(e){seen=seen+'a';e.stopPropagation();},true);\
         p.addEventListener('click',function(){seen=seen+'b';},true);\
         c.addEventListener('click',function(){seen=seen+'target';});\
         p.addEventListener('click',function(){seen=seen+'bubble';});\
         c.dispatchEvent({type:'click',bubbles:true});seen;",
    );

    assert_eq!(value, JsValue::String("ab".into()));
}

#[test]
fn stop_immediate_propagation_skips_same_capture_target_listeners() {
    let value = eval(
        "let p=document.getElementById('p');let c=document.getElementById('c');\
         let seen='';\
         p.addEventListener('click',function(e){seen=seen+'a';e.stopImmediatePropagation();},true);\
         p.addEventListener('click',function(){seen=seen+'b';},true);\
         c.dispatchEvent({type:'click',bubbles:true});seen;",
    );

    assert_eq!(value, JsValue::String("a".into()));
}

#[test]
fn stop_propagation_preserves_same_bubble_target_listeners() {
    let value = eval(
        "let p=document.getElementById('p');let c=document.getElementById('c');\
         let seen='';\
         c.addEventListener('click',function(){seen=seen+'target';});\
         p.addEventListener('click',function(e){seen=seen+'a';e.stopPropagation();});\
         p.addEventListener('click',function(){seen=seen+'b';});\
         c.dispatchEvent({type:'click',bubbles:true});seen;",
    );

    assert_eq!(value, JsValue::String("targetab".into()));
}

#[test]
fn composed_shadow_events_reach_host_with_path() {
    let result = eval_with_dom(
        "<div id='host'></div>",
        "let host=document.getElementById('host');\
         let root=host.attachShadow({mode:'open'});let button=document.createElement('button');\
         button.setAttribute('id','inside');root.appendChild(button);let seen='';\
         host.addEventListener('click',function(e){let p=e.composedPath();\
         seen=p[0].id+':'+p[2].id+':'+e.currentTarget.id+':'+e.eventPhase;});\
         button.dispatchEvent({type:'click',bubbles:true,composed:true});seen;",
    )
    .unwrap()
    .value;

    assert_eq!(result, JsValue::String("inside:host:host:3".into()));
}
