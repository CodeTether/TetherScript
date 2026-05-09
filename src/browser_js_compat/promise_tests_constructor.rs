use super::super::*;

#[test]
fn promise_constructor_resolve_path_runs_handler() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let out='';new P(function(resolve,reject){resolve('ok');}).then(function(v){out=v;});out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("ok".into()));
}

#[test]
fn promise_constructor_reject_path_runs_handler() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let out='';P(function(resolve,reject){reject('no');}).catch(function(e){out=e;});out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("no".into()));
}
