use super::case::{assert_dom, Case, DomCase};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: DomCase = DomCase {
    case: Case {
        area: "dom/events",
        wpt_shape: "dispatchEvent capture/target/bubble order and default prevention",
        unsupported: &["unforgeable trust boundary", "complete UIEvent subclasses"],
    },
    html: "<div id='p'><button id='c'>Go</button></div>",
    script: "let p=document.getElementById('p');let c=document.getElementById('c');\
        let seen='';p.addEventListener('click',function(e){seen=seen+'capture:'+e.eventPhase+'>';},true);\
        c.addEventListener('click',function(e){seen=seen+'target:'+e.currentTarget.id+'>';e.preventDefault();});\
        p.addEventListener('click',function(e){seen=seen+'bubble:'+e.defaultPrevented;});\
        let ok=c.dispatchEvent({type:'click',bubbles:true,cancelable:true});seen+':'+ok;",
    expect: "capture:1>target:c>bubble:true:false",
};

pub fn run() {
    assert_dom(&CASE);
    native_click_is_trusted();
}

fn native_click_is_trusted() {
    let html = "<button id='go'>Go</button><script style='display:none'>\
        let b=document.getElementById('go');window.trust='';\
        b.addEventListener('click',function(e){window.trust+=e.isTrusted+'>';});\
        b.dispatchEvent(Event('click'));</script>";
    let mut page = BrowserPage::from_html("mem://trusted-click", html);

    page.click(&Locator::css("#go")).unwrap();

    assert_eq!(
        page.eval_js("window.trust").unwrap().display(),
        "false>true>"
    );
}
