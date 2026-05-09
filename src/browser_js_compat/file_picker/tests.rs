use super::super::*;

#[test]
fn picker_functions_are_window_and_bare_globals() {
    let result = eval_with_dom(
        "<main></main>",
        "typeof showOpenFilePicker+'|'+(showOpenFilePicker===window.showOpenFilePicker)+'|'\
         +typeof showSaveFilePicker+'|'+typeof showDirectoryPicker;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("function|true|function|function".into())
    );
}

#[test]
fn picker_calls_return_synchronous_rejected_thenables() {
    let result = eval_with_dom(
        "<main></main>",
        "let seen='';let p=showOpenFilePicker();\
         let next=p.catch(function(e){seen=e.name+':'+e.message;return 'handled';});\
         [p.__promise_state,seen,next.__promise_state,next.__promise_value].join('|');",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String(
            "rejected|NotAllowedError:file picker unsupported|fulfilled|handled".into()
        )
    );
}

#[test]
fn picker_then_without_rejection_handler_stays_rejected() {
    let result = eval_with_dom(
        "<main></main>",
        "let p=showSaveFilePicker();let next=p.then(null);\
         next.__promise_state+'|'+next.__promise_reason.name;",
    )
    .unwrap();
    assert_eq!(
        result.value,
        JsValue::String("rejected|NotAllowedError".into())
    );
}
