use super::*;

fn eval_nested(script: &str) -> JsValue {
    eval_with_dom(
        "<div id='outer'><div id='inner'><button id='btn'></button></div></div>",
        script,
    )
    .unwrap()
    .value
}

#[test]
fn capture_target_bubble_ordering() {
    let script = "let btn=document.getElementById('btn');let seen='';\
        document.getElementById('outer').addEventListener('click',function(){seen=seen+'OC;';},true);\
        document.getElementById('inner').addEventListener('click',function(){seen=seen+'IC;';},true);\
        document.getElementById('inner').addEventListener('click',function(){seen=seen+'IB;';},false);\
        document.getElementById('outer').addEventListener('click',function(){seen=seen+'OB;';},false);\
        btn.click();seen;";
    assert_eq!(eval_nested(script).display(), "OC;IC;IB;OB;");
}

#[test]
fn stop_propagation_blocks_later_phases() {
    let script = "let btn=document.getElementById('btn');let seen='';\
        document.getElementById('outer').addEventListener('click',function(){seen=seen+'OC;';},true);\
        document.getElementById('inner').addEventListener('click',function(e){seen=seen+'IC;';e.stopPropagation();},true);\
        document.getElementById('inner').addEventListener('click',function(){seen=seen+'IB;';},false);\
        document.getElementById('outer').addEventListener('click',function(){seen=seen+'OB;';},false);\
        btn.click();seen;";
    assert_eq!(eval_nested(script).display(), "OC;IC;");
}

#[test]
fn prevent_default_suppresses_default_action() {
    let script = "let btn=document.getElementById('btn');let seen='';\
        btn.addEventListener('click',function(e){seen=seen+'T;';e.preventDefault();});\
        let result=btn.click();seen+':'+result;";
    assert_eq!(eval_nested(script).display(), "T;:false");
}

#[test]
fn prevent_default_allows_propagation() {
    let script = "let btn=document.getElementById('btn');let seen='';\
        document.getElementById('outer').addEventListener('click',function(){seen=seen+'OB;';});\
        btn.addEventListener('click',function(e){seen=seen+'T;';e.preventDefault();});\
        btn.click();seen;";
    assert_eq!(eval_nested(script).display(), "T;OB;");
}
