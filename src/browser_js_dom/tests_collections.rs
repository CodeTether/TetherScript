use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[path = "tests_html_collection_named.rs"]
mod tests_html_collection_named;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn query_selector_all_exposes_item_and_for_each() {
    assert_eq!(
        eval(
            "<ul><li>A</li><li>B</li></ul>",
            "let nodes=document.querySelectorAll('li');let seen='';\
         nodes.forEach(function(n,i,c){seen=seen+i+n.textContent+(c===nodes);});\
         nodes.length+':'+nodes[1].textContent+':'+nodes.item(0).textContent\
         +':'+nodes.item(2)+':'+seen;",
        ),
        JsValue::String("2:B:A:null:0Atrue1Btrue".into())
    );
}

#[test]
fn children_collection_updates_after_mutation() {
    assert_eq!(
        eval(
            "<div id='app'><span>A</span><em>B</em></div>",
            "let app=document.getElementById('app');let kids=app.children;\
         let strong=document.createElement('strong');app.appendChild(strong);\
         let seen='';kids.forEach(function(n,i,c){seen=seen+i+n.tagName+(c===kids);});\
         kids.length+':'+app.children.length+':'+kids[0].textContent\
         +':'+kids.item(1).tagName+':'+kids[2].tagName+':'+seen;",
        ),
        JsValue::String("3:3:A:EM:STRONG:0SPANtrue1EMtrue2STRONGtrue".into())
    );
}

#[test]
fn child_nodes_collection_includes_text_nodes() {
    assert_eq!(
        eval(
            "<div id='app'>A<span>B</span>C</div>",
            "let app=document.getElementById('app');let nodes=app.childNodes;\
         app.appendChild(document.createTextNode('D'));let seen='';\
         nodes.forEach(function(n,i,c){seen=seen+i+n.nodeType+(c===nodes);});\
         nodes.length+':'+nodes[0].textContent+':'+nodes.item(1).tagName\
         +':'+nodes.item(3).textContent+':'+seen;",
        ),
        JsValue::String("4:A:SPAN:D:03true11true23true33true".into())
    );
}
