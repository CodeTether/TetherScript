use super::super::*;

#[test]
fn clipboard_item_is_global_and_exposes_types() {
    let result = eval_with_dom(
        "<main></main>",
        "let item=ClipboardItem({'text/plain':'p','text/html':'h'}); \
         typeof ClipboardItem+'|'+(ClipboardItem===window.ClipboardItem)+'|'+item.types.join(',');",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("function|true|text/html,text/plain".into())
    );
}

#[test]
fn clipboard_item_get_type_returns_blob_text_and_type() {
    let result = eval_with_dom(
        "<main></main>",
        "let out=''; let item=ClipboardItem({'text/plain':'hello'}); \
         item.getType('text/plain').then(function(blob){ \
         blob.text().then(function(text){ out=text+'|'+blob.type+'|'+blob.size; }); }); out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("hello|text/plain|5".into()));
}

#[test]
fn clipboard_item_preserves_blob_bytes_and_type() {
    let result = eval_with_dom(
        "<main></main>",
        "let out=''; let source=Blob(['AB'],{type:'text/custom'}); \
         let item=ClipboardItem({'image/png':source}); \
         item.getType('image/png').then(function(blob){ \
         blob.text().then(function(text){ out=text+'|'+blob.type+'|'+blob.size; }); }); out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("AB|text/custom|2".into()));
}

#[test]
fn clipboard_item_get_type_rejects_missing_type() {
    let result = eval_with_dom(
        "<main></main>",
        "let out=''; let item=ClipboardItem({'text/plain':'hello'}); \
         item.getType('image/png').catch(function(error){ out=error; }); out;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("ClipboardItem: missing type image/png".into())
    );
}
