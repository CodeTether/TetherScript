use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn element_buffer_draws_indexed_triangles_into_real_pixels() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
         let v=s(gl.VERTEX_SHADER,'attribute vec2 p;void main(){gl_Position=vec4(p,0.0,1.0); }');\
         let f=s(gl.FRAGMENT_SHADER,'void main(){gl_FragColor=vec4(0.0,0.0,1.0,1.0); }');\
         let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
         let vb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,vb);\
         gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,1,1,-1,1]),gl.STATIC_DRAW);\
         let a=gl.getAttribLocation(p,'p');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,0,0);\
         let eb=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,eb);\
         gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,12,gl.DYNAMIC_DRAW);\
         gl.bufferSubData(gl.ELEMENT_ARRAY_BUFFER,0,new Uint16Array([0,1,2,0,2,3]));\
         gl.drawElements(gl.TRIANGLES,6,gl.UNSIGNED_SHORT,0);\
         let size=gl.getBufferParameter(gl.ELEMENT_ARRAY_BUFFER,gl.BUFFER_SIZE);\
         gl.clear(gl.COLOR_BUFFER_BIT);gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint8Array([0,1,2]),gl.STATIC_DRAW);\
         gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_BYTE,0);let pixel=new Uint8Array(4);\
         gl.readPixels(1,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);\
         [gl.getParameter(gl.ARRAY_BUFFER_BINDING)===vb,gl.getParameter(gl.ELEMENT_ARRAY_BUFFER_BINDING)===eb,\
          size,pixel.join(','),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("true|true|12|0,0,255,255|0".into())
    );
}
