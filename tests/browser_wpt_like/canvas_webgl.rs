use super::case::{assert_case, Case};
use tetherscript::browser_agent::{BrowserPage, Locator};

const CASE: Case = Case {
    area: "html/canvas/offscreen/webgl",
    wpt_shape: "2D ImageData and WebGL color operations update native raster buffers",
    unsupported: &["WebGL shaders, buffers, textures, and draw calls"],
};

pub fn run() {
    assert_case(&CASE);
    let html = "<canvas id='c' width='4' height='3'></canvas>\
        <canvas id='gl' width='8' height='4'></canvas>";
    let script = "let c=document.getElementById('c');let ctx=c.getContext('2d');\
        ctx.fillStyle='#f00';ctx.fillRect(1,1,2,1);\
        let image=ctx.createImageData(1,1);image.data.set([0,0,255,255]);\
        ctx.putImageData(image,0,0);let canvasPixels=ctx.getImageData(0,0,1,1).data.join(',');\
        let gl=document.getElementById('gl').getContext('webgl2');\
        gl.viewport(1,2,3,4);gl.clearColor(1,0,0,1);gl.clear(gl.COLOR_BUFFER_BIT);\
        let pixels=new Uint8Array(4);\
        gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixels);\
        gl.enable(gl.SCISSOR_TEST);gl.scissor(1,0,2,1);gl.colorMask(false,true,true,true);\
        gl.clearColor(0,1,1,1);gl.clear(gl.COLOR_BUFFER_BIT);let masked=new Uint8Array(4);\
        gl.readPixels(1,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,masked);\
        canvasPixels+'|'+pixels.join(',')+'|'+masked.join(',');";
    let mut page = BrowserPage::from_html("mem://canvas", html);
    let readback = page.eval_js(script).unwrap();
    let surface = page.canvas_surface(&Locator::css("#c")).unwrap();
    let webgl = page.webgl_context(&Locator::css("#gl")).unwrap();
    assert_eq!((surface.width, surface.height), (4, 3));
    assert_eq!(surface.commands[0].operation, "fillRect");
    assert_eq!(surface.commands[0].args, vec![1, 1, 2, 1]);
    assert_eq!((webgl.version, webgl.width, webgl.height), (2, 8, 4));
    assert_eq!(webgl.viewport, [1, 2, 3, 4]);
    assert_eq!(webgl.scissor_box, [1, 0, 2, 1]);
    assert!(webgl.scissor_test);
    assert_eq!(webgl.color_mask, [false, true, true, true]);
    assert_eq!(webgl.commands[2].operation, "clear");
    assert_eq!(webgl.commands.last().unwrap().operation, "clear");
    let gl_surface = page.canvas_surface(&Locator::css("#gl")).unwrap();
    let red = u32::from_be_bytes([255, 0, 0, 255]) as u64;
    let white = u32::from_be_bytes([255, 255, 255, 255]) as u64;
    assert_eq!(
        gl_surface.checksum,
        Some(528 * (red + 1) + 53 * (white - red))
    );
    assert_eq!(
        readback.display(),
        "0,0,255,255|255,0,0,255|255,255,255,255"
    );
}
