use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn unpack_flip_and_premultiply_transform_sampled_texture_pixels() {
    let result = eval_with_dom(
        "<canvas id='gl' width='2' height='2'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
         let v=s(gl.VERTEX_SHADER,'attribute vec2 position;attribute vec2 uv;varying vec2 point;void main(){gl_Position=vec4(position,0.0,1.0);point=uv;}');\
         let f=s(gl.FRAGMENT_SHADER,'varying vec2 point;uniform sampler2D image;void main(){gl_FragColor=texture2D(image,point);}');\
         let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
         let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(\
         [-1,-1,0,0,1,-1,1,0,1,1,1,1,-1,-1,0,0,1,1,1,1,-1,1,0,1]),gl.STATIC_DRAW);\
         for(let x of [['position',0],['uv',8]]){let a=gl.getAttribLocation(p,x[0]);gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,16,x[1]);}\
         let image=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,image);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);\
         gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,true);gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL,true);\
         let source=new ImageData(new Uint8ClampedArray([200,100,50,128,0,0,255,255]),1,2);\
         gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,source);\
         gl.uniform1i(gl.getUniformLocation(p,'image'),0);gl.drawArrays(gl.TRIANGLES,0,6);\
         function px(y){let q=new Uint8Array(4);gl.readPixels(0,y,1,1,gl.RGBA,gl.UNSIGNED_BYTE,q);return q.join(',');}\
         [gl.getParameter(gl.UNPACK_FLIP_Y_WEBGL),gl.getParameter(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL),px(0),px(1),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("true|true|0,0,255,255|100,50,25,128|0".into())
    );
}
