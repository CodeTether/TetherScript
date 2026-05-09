use super::super::super::super::*;

fn run(script: &str) -> JsValue {
    eval_with_dom("<main></main>", script).unwrap().value
}

#[test]
fn scheduler_is_window_and_bare_global() {
    let result = run("typeof scheduler + ':' + (scheduler === window.scheduler);");
    assert_eq!(result, JsValue::String("object:true".into()));
}

#[test]
fn post_task_runs_callback_and_resolves_return_value() {
    let result = run("let out=''; scheduler.postTask(function(){ return 7; },\
         {priority:'user-visible'}).then(function(v){ out='v'+v; }); out;");
    assert_eq!(result, JsValue::String("v7".into()));
}

#[test]
fn post_task_without_callback_resolves_undefined() {
    let result = run("let out=''; scheduler.postTask().then(function(v){ out=typeof v; }); out;");
    assert_eq!(result, JsValue::String("undefined".into()));
}

#[test]
fn yield_returns_resolved_thenable() {
    let result = run("let out=''; scheduler.yield().then(function(v){ out=typeof v; }); out;");
    assert_eq!(result, JsValue::String("undefined".into()));
}
