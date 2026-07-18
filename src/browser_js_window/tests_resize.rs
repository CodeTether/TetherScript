use super::super::eval_with_dom;
use crate::js::JsValue;

#[test]
fn resize_to_updates_window_size_aliases_and_dispatches() {
    let result = eval_with_dom(
        "<main></main>",
        "let seen='';window.onresize=function(e){seen=e.type+':' +(e.target===window)+':' +innerWidth+'x'+innerHeight;};resizeTo(1024,768);[window.innerWidth,innerHeight,seen].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("1024|768|resize:true:1024x768".into())
    );
}

#[test]
fn resize_updates_legacy_orientation_and_dispatches_change() {
    let result = eval_with_dom(
        "<main></main>",
        "let seen='';window.onorientationchange=function(e){seen=e.type+':' +(e.target===window)+':' +window.orientation;};resizeTo(10,200);[orientation,window.orientation,screen.orientation.angle,seen].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("90|90|90|orientationchange:true:90".into())
    );
}
