use super::super::*;

#[test]
fn promise_all_unwraps_mixed_values_and_thenables() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let out='';P.all([1,P.resolve(2),Blob(['x']).text()]).then(function(v){out=v.join('-');});out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("1-2-x".into()));
}

#[test]
fn promise_all_rejects_when_any_input_is_rejected() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let out='';P.all([1,P.reject('no'),P.resolve(3)]).catch(function(e){out=e;});out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("no".into()));
}

#[test]
fn promise_race_settles_to_first_item() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let a='';let b='';P.race(['first',P.resolve('later')]).then(function(v){a=v;});P.race([P.reject('bad'),P.resolve('ok')]).catch(function(e){b=e;});a+':'+b;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("first:bad".into()));
}
