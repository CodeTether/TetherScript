use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

fn eval(html: &str, script: &str) -> JsValue {
    eval_with_dom(html, script).unwrap().value
}

#[test]
fn get_elements_by_tag_updates_after_append() {
    assert_eq!(
        eval(
            "<main><span id='a'></span></main>",
            "let spans=document.getElementsByTagName('span');\
             let next=document.createElement('span');next.id='b';\
             document.querySelector('main').appendChild(next);\
             spans.length+':'+spans[1].id+':'+spans.item(1).id;",
        ),
        JsValue::String("2:b:b".into())
    );
}

#[test]
fn class_and_name_collections_update_after_attribute_changes() {
    assert_eq!(
        eval(
            "<input id='a' class='one' name='old'><input id='b'>",
            "let byClass=document.getElementsByClassName('one two');\
             let byName=document.getElementsByName('q');let b=document.getElementById('b');\
             b.className='two one';b.name='q';\
             byClass.length+':'+byClass[0].id+':'+byName.length+':'+byName[0].id;",
        ),
        JsValue::String("1:b:1:b".into())
    );
}

#[test]
fn document_named_collections_stay_live() {
    assert_eq!(
        eval(
            "<form id='login'></form>",
            "let forms=document.forms;let images=document.images;\
             let f=document.createElement('form');f.id='late';document.body.appendChild(f);\
             let img=document.createElement('img');img.name='hero';document.body.appendChild(img);\
             forms.length+':'+forms.namedItem('late').id+':'+forms.late.id\
             +':'+images.length+':'+images.namedItem('hero').name;",
        ),
        JsValue::String("2:late:late:1:hero".into())
    );
}
