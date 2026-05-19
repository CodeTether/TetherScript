use super::case::{assert_dom, Case, DomCase};

const CASE: DomCase = DomCase {
    case: Case {
        area: "dom/events",
        wpt_shape: "dispatchEvent capture/target/bubble order and default prevention",
        unsupported: &["trusted event flags", "complete UIEvent subclasses"],
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
}
