use tetherscript::browser_agent::{BrowserPage, Locator};

pub(super) fn run() {
    let html = "<canvas id='gl' width='4' height='4'></canvas>";
    let script = "let gl=document.getElementById('gl').getContext('webgl');\
        function shader(t,s){let x=gl.createShader(t);gl.shaderSource(x,s);gl.compileShader(x);return x;}\
        let vs=shader(gl.VERTEX_SHADER,'attribute vec2 position;void main(){gl_Position=vec4(position,0.0,1.0); }');\
        let fs=shader(gl.FRAGMENT_SHADER,'void main(){gl_FragColor=vec4(1.0,0.0,0.0,1.0); }');\
        let p=gl.createProgram();gl.attachShader(p,vs);gl.attachShader(p,fs);gl.linkProgram(p);gl.useProgram(p);\
        let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);\
        gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,0,1]),gl.STATIC_DRAW);\
        let a=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,0,0);\
        gl.drawArrays(gl.TRIANGLES,0,3);let pixel=new Uint8Array(4);\
        gl.readPixels(2,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);pixel.join(',');";
    let mut page = BrowserPage::from_html("mem://webgl-draw", html);
    let readback = page.eval_js(script).unwrap();
    let webgl = page.webgl_context(&Locator::css("#gl")).unwrap();

    assert_eq!(readback.display(), "255,0,0,255");
    assert_eq!(webgl.commands.last().unwrap().operation, "drawArrays");
    assert_eq!(webgl.commands.last().unwrap().args, vec!["4", "0", "3"]);
}
