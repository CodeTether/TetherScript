use super::case::{assert_case, Case};
use tetherscript::browser_agent::permissions::BrowserPermission;
use tetherscript::browser_agent::BrowserPage;
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "permissions/media-capture",
    wpt_shape: "permission grants expose media labels and allow getUserMedia",
    unsupported: &[
        "real device capture",
        "constraint solving and live MediaStream tracks",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test", "");
    page.grant_permission("https://app.test", BrowserPermission::Camera);
    page.grant_permission("https://app.test", BrowserPermission::Microphone);
    let script = "let out='';\
        navigator.permissions.query({name:'camera'}).then(function(p){out=p.state;});\
        navigator.mediaDevices.enumerateDevices().then(function(d){\
        out=out+':'+d[0].label+'|'+d[1].label;});\
        navigator.mediaDevices.getUserMedia({video:true,audio:true}).then(function(s){\
        out=out+':'+s.active+':'+s.constraints.video+':'+s.constraints.audio;});out;";
    assert_eq!(
        page.eval_js(script).unwrap(),
        JsValue::String("granted:Agent Camera|Agent Microphone:true:true:true".into())
    );
}
