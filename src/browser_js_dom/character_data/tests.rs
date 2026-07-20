use super::super::super::*;

#[test]
fn text_character_data_methods_use_utf16_offsets() {
    let result = eval_with_dom(
        "<p id='x'>A😀BC</p>",
        "let t=document.getElementById('x').firstChild;\
         let before=t.data+':'+t.length+':'+t.substringData(1,2);\
         t.deleteData(1,2);t.insertData(1,'x');t.replaceData(2,1,'yz');\
         t.appendData('!');before+'|'+t.data+':'+t.nodeValue;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("A😀BC:5:😀|AxyzC!:AxyzC!".into())
    );
}

#[test]
fn character_data_mutation_records_old_value() {
    let result = eval_with_dom(
        "<p id='x'>old</p>",
        "let t=document.getElementById('x').firstChild;\
         let o=MutationObserver(r=>console.log(r[0].type+':'+r[0].oldValue));\
         o.observe(t,{characterData:true,characterDataOldValue:true});\
         t.data='new';'sync';",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("sync".into()));
    assert_eq!(result.console, vec!["characterData:old".to_string()]);
}

#[test]
fn character_data_rejects_offsets_past_length() {
    let result = eval_with_dom(
        "<p id='x'>a</p>",
        "document.getElementById('x').firstChild.deleteData(2,1);",
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("out-of-range CharacterData offset must fail"),
    };
    assert!(error.contains("IndexSizeError: CharacterData.deleteData offset 2 exceeds length 1"));
}
