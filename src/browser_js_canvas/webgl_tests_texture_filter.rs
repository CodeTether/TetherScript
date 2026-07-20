use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn texture_minification_filters_linearly_and_repeat_wraps_coordinates() {
    let result = eval_with_dom(
        "<canvas id='gl' width='1' height='1'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
         let v=s(gl.VERTEX_SHADER,'attribute vec2 position;attribute vec2 uv;varying vec2 point;void main(){gl_Position=vec4(position,0.0,1.0);point=uv;}');\
         let f=s(gl.FRAGMENT_SHADER,'varying vec2 point;uniform sampler2D image;void main(){gl_FragColor=texture2D(image,point);}');\
         let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
         let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);let first=new Float32Array(\
         [-1,-1,0,0,1,-1,1,0,1,1,1,1,-1,-1,0,0,1,1,1,1,-1,1,0,1]);gl.bufferData(gl.ARRAY_BUFFER,first,gl.DYNAMIC_DRAW);\
         let a=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,16,0);\
         let u=gl.getAttribLocation(p,'uv');gl.enableVertexAttribArray(u);gl.vertexAttribPointer(u,2,gl.FLOAT,false,16,8);\
         let image=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,image);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);\
         gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.REPEAT);\
         gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.REPEAT);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,2,2,0,gl.RGBA,gl.UNSIGNED_BYTE,\
         new Uint8Array([255,0,0,255,0,255,0,255,0,0,255,255,255,255,0,255]));gl.uniform1i(gl.getUniformLocation(p,'image'),0);\
         function pixel(){let q=new Uint8Array(4);gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,q);return q.join(',');}\
         gl.drawArrays(gl.TRIANGLES,0,6);let linear=pixel();let second=[];for(let i=0;i<6;i++){second.push(first[i*4],first[i*4+1],1.25,0.25);}\
         gl.bufferSubData(gl.ARRAY_BUFFER,0,new Float32Array(second));gl.drawArrays(gl.TRIANGLES,0,6);[linear,pixel(),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("128,128,64,255|255,0,0,255|0".into())
    );
}
