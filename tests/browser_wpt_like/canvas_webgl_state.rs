use tetherscript::browser_agent::{BrowserPage, Locator};

pub(super) fn run() {
    let html = "<canvas id='gl' width='8' height='4'></canvas>";
    let script = "let gl=document.getElementById('gl').getContext('webgl2');\
        gl.viewport(1,2,3,4);gl.clearColor(1,0,0,1);gl.clear(gl.COLOR_BUFFER_BIT);\
        let pixels=new Uint8Array(4);gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixels);\
        gl.enable(gl.SCISSOR_TEST);gl.scissor(1,0,2,1);gl.colorMask(false,true,true,true);\
        gl.clearColor(0,1,1,1);gl.clear(gl.COLOR_BUFFER_BIT);let masked=new Uint8Array(4);\
        gl.readPixels(1,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,masked);pixels.join(',')+'|'+masked.join(',');";
    let mut page = BrowserPage::from_html("mem://webgl-state", html);
    let readback = page.eval_js(script).unwrap();
    let webgl = page.webgl_context(&Locator::css("#gl")).unwrap();

    assert_eq!((webgl.version, webgl.width, webgl.height), (2, 8, 4));
    assert_eq!(webgl.viewport, [1, 2, 3, 4]);
    assert_eq!(webgl.scissor_box, [1, 0, 2, 1]);
    assert!(webgl.scissor_test);
    assert_eq!(webgl.color_mask, [false, true, true, true]);
    assert_eq!(webgl.commands.last().unwrap().operation, "clear");
    assert_eq!(readback.display(), "255,0,0,255|255,255,255,255");
}
