use tetherscript::browser_agent::{BrowserPage, Locator};

pub(super) fn run() {
    let html = "<canvas id='gl' width='4' height='4'></canvas>";
    let script = "let gl=document.getElementById('gl').getContext('webgl');\
        function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
        let v=s(gl.VERTEX_SHADER,'attribute vec2 position;attribute vec2 uv;varying vec2 point;void main(){gl_Position=vec4(position,0.0,1.0);point=uv;}');\
        let f=s(gl.FRAGMENT_SHADER,'varying vec2 point;uniform sampler2D image;void main(){gl_FragColor=texture2D(image,point);}');\
        let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
        let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(\
        [-1,-1,0,0,1,-1,1,0,1,1,1,1,-1,-1,0,0,1,1,1,1,-1,1,0,1]),gl.STATIC_DRAW);\
        let a=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,16,0);\
        let u=gl.getAttribLocation(p,'uv');gl.enableVertexAttribArray(u);gl.vertexAttribPointer(u,2,gl.FLOAT,false,16,8);\
        let image=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,image);\
        gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.NEAREST);\
        gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,2,2,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(\
        [255,0,0,255,0,255,0,255,0,0,255,255,255,255,0,255]));\
        gl.uniform1i(gl.getUniformLocation(p,'image'),0);gl.drawArrays(gl.TRIANGLES,0,6);let pixel=new Uint8Array(4);\
        gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);pixel.join(',');";
    let mut page = BrowserPage::from_html("mem://webgl-texture", html);
    let readback = page.eval_js(script).unwrap();
    let webgl = page.webgl_context(&Locator::css("#gl")).unwrap();

    assert_eq!(readback.display(), "255,0,0,255");
    assert_eq!(webgl.commands.last().unwrap().operation, "drawArrays");
}
