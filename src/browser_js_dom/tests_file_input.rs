use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn file_input_files_empty_without_upload() {
    let result = eval_with_dom(
        "<input id='u' type='file'>",
        "let f=document.getElementById('u').files; f.length + ':' + f.item(0) + ':' + (f[0] === undefined);",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("0:null:true".into()));
}

#[test]
fn file_input_files_expose_single_file_fields() {
    let html = "<input id='u' type='file' data-agent-files='[{\"name\":\"note.txt\",\"type\":\"text/plain\",\"size\":3}]'>";
    let result = eval_with_dom(html, "let f=document.getElementById('u').files; let a=f[0]; f.length+':'+f.item(0).name+':'+a.name+':'+a.type+':'+a.size+':'+a.lastModified").unwrap();
    assert_eq!(
        result.value,
        JsValue::String("1:note.txt:note.txt:text/plain:3:0".into())
    );
}

#[test]
fn file_input_files_expose_multiple_indices_and_item() {
    let html = "<input id='u' type='file' data-agent-files='[{\"name\":\"a.txt\",\"type\":\"text/plain\",\"size\":1},{\"name\":\"b.bin\",\"type\":\"application/octet-stream\",\"size\":2}]'>";
    let result = eval_with_dom(
        html,
        "let f=document.getElementById('u').files; f.length+':'+f[0].name+':'+f.item(1).name+':'+f.item(2)",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("2:a.txt:b.bin:null".into()));
}
