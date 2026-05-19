use super::case::{assert_case, Case};
use tetherscript::browser_js::eval_with_dom;
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "html/webappapis/timers",
    wpt_shape: "microtasks, animation frames, and timers drain deterministically",
    unsupported: &["wall-clock scheduling", "task-source prioritization matrix"],
};

pub fn run() {
    assert_case(&CASE);
    let result = eval_with_dom(
        "<main></main>",
        "let order='';setTimeout(function(){order=order+'T';console.log(order);},0);\
         requestAnimationFrame(function(){order=order+'R';});\
         queueMicrotask(function(){order=order+'M';});order=order+'S';'sync';",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("sync".into()));
    assert_eq!(result.console, vec!["SMRT".to_string()]);
}
