use super::case::{assert_case, Case};
use tetherscript::browser_agent::permissions::BrowserPermission;
use tetherscript::browser_agent::BrowserPage;
use tetherscript::js::JsValue;

const CASE: Case = Case {
    area: "permissions/media-capture",
    wpt_shape: "denied camera permission rejects getUserMedia with NotAllowedError",
    unsupported: &["permission prompt UI", "device-specific denial reasons"],
};

pub fn run() {
    assert_case(&CASE);
    let mut page = BrowserPage::from_html("https://app.test", "");
    page.deny_permission("https://app.test", BrowserPermission::Camera);
    let script = "let out='';navigator.permissions.query({name:'camera'}).then(function(p){\
        out=p.state;});navigator.mediaDevices.getUserMedia({video:true}).catch(function(e){\
        out=out+':'+e.name+':'+e.message;});out;";
    assert_eq!(
        page.eval_js(script).unwrap(),
        JsValue::String("denied:NotAllowedError:camera denied".into())
    );
}
