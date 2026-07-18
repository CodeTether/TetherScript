use tetherscript::browser_agent::{BrowserPage, Locator};

pub(super) fn run() {
    let html = "<canvas id='c' width='4' height='3'></canvas>";
    let script = "let c=document.getElementById('c');let ctx=c.getContext('2d');\
        ctx.fillStyle='#f00';ctx.fillRect(1,1,2,1);\
        let image=ctx.createImageData(1,1);image.data.set([0,0,255,255]);\
        ctx.putImageData(image,0,0);ctx.getImageData(0,0,1,1).data.join(',');";
    let mut page = BrowserPage::from_html("mem://canvas-2d", html);
    let readback = page.eval_js(script).unwrap();
    let surface = page.canvas_surface(&Locator::css("#c")).unwrap();

    assert_eq!((surface.width, surface.height), (4, 3));
    assert_eq!(surface.commands[0].operation, "fillRect");
    assert_eq!(surface.commands[0].args, vec![1, 1, 2, 1]);
    assert_eq!(readback.display(), "0,0,255,255");
}
