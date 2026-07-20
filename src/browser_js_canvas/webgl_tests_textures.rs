use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn typed_rgba_texture_uploads_and_samples_real_fragments() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
         let v=s(gl.VERTEX_SHADER,'attribute vec2 position;attribute vec2 uv;varying vec2 point;void main(){gl_Position=vec4(position,0.0,1.0);point=uv;}');\
         let f=s(gl.FRAGMENT_SHADER,'precision mediump float;varying vec2 point;uniform sampler2D image;void main(){gl_FragColor=texture2D(image,point);}');\
         let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
         let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(\
         [-1,-1,0,0,1,-1,1,0,1,1,1,1,-1,-1,0,0,1,1,1,1,-1,1,0,1]),gl.STATIC_DRAW);\
         let pos=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(pos);gl.vertexAttribPointer(pos,2,gl.FLOAT,false,16,0);\
         let uv=gl.getAttribLocation(p,'uv');gl.enableVertexAttribArray(uv);gl.vertexAttribPointer(uv,2,gl.FLOAT,false,16,8);\
         let image=gl.createTexture();gl.activeTexture(gl.TEXTURE0+1);gl.bindTexture(gl.TEXTURE_2D,image);\
         gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);\
         gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);\
         gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,2,2,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(\
         [255,0,0,255,0,255,0,255,0,0,255,255,255,255,0,255]));\
         let patch=new ImageData(new Uint8ClampedArray([255,0,255,255]),1,1);\
         gl.texSubImage2D(gl.TEXTURE_2D,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,patch);\
         gl.uniform1i(gl.getUniformLocation(p,'image'),1);gl.drawArrays(gl.TRIANGLES,0,6);\
         function px(x,y){let q=new Uint8Array(4);gl.readPixels(x,y,1,1,gl.RGBA,gl.UNSIGNED_BYTE,q);return q.join(',');}\
         [gl.isTexture(image),gl.getParameter(gl.TEXTURE_BINDING_2D)===image,gl.getParameter(gl.ACTIVE_TEXTURE),\
          gl.getTexParameter(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER),gl.getProgramParameter(p,gl.ACTIVE_UNIFORMS),\
          gl.getParameter(gl.MAX_TEXTURE_SIZE),gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS),\
          px(0,0),px(3,0),px(0,3),px(3,3),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String(
            "true|true|33985|9728|1|2048|8|255,0,0,255|0,255,0,255|0,0,255,255|255,0,255,255|0"
                .into()
        )
    );
}
