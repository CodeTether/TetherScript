use super::super::*;

#[test]
fn promise_resolve_then_runs_synchronously() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let seen='';P.resolve(4).then(function(v){seen='r'+v;});seen;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("r4".into()));
}

#[test]
fn promise_reject_catch_runs_synchronously() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let seen='';P.reject('bad').catch(function(e){seen=e;});seen;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("bad".into()));
}

#[test]
fn promise_finally_preserves_original_settlement() {
    let result = eval_with_dom(
        "<main></main>",
        "let P=window.Promise;let a='';let b='';P.resolve('ok').finally(function(){a='f';}).then(function(v){a=a+v;});P.reject('no').finally(function(){b='f';}).catch(function(e){b=b+e;});a+':'+b;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("fok:fno".into()));
}
