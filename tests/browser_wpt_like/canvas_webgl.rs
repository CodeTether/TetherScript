use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/canvas/offscreen/webgl",
    wpt_shape: "2D canvas command logs and WebGL metadata snapshots are observable",
    unsupported: &[
        "GPU rendering",
        "complete CanvasRenderingContext2D and WebGL APIs",
    ],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<canvas id='c' width='4' height='3'></canvas>\
        <canvas id='gl' width='8' height='4'></canvas>";
    let script = "let c=document.getElementById('c');let ctx=c.getContext('2d');\
        ctx.fillStyle='#f00';ctx.fillRect(1,1,2,1);\
        let gl=document.getElementById('gl').getContext('webgl2');\
        gl.viewport(1,2,3,4);gl.clear(gl.COLOR_BUFFER_BIT);";
    let mut page = BrowserPage::from_html("mem://canvas", html);
    page.eval_js(script).unwrap();
    let surface = page.canvas_surface(&Locator::css("#c")).unwrap();
    let webgl = page.webgl_context(&Locator::css("#gl")).unwrap();
    assert_eq!((surface.width, surface.height), (4, 3));
    assert_eq!(surface.commands[0].operation, "fillRect");
    assert_eq!(surface.commands[0].args, vec![1, 1, 2, 1]);
    assert_eq!((webgl.version, webgl.width, webgl.height), (2, 8, 4));
    assert_eq!(webgl.viewport, [1, 2, 3, 4]);
    assert_eq!(webgl.commands[1].operation, "clear");
}
