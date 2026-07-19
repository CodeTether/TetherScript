use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn webgl2_draw_elements_decodes_unsigned_int_indices() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl2');\
         function s(t,x){let q=gl.createShader(t);gl.shaderSource(q,x);gl.compileShader(q);return q;}\
         let v=s(gl.VERTEX_SHADER,'in vec2 p;void main(){gl_Position=vec4(p,0.0,1.0); }');\
         let f=s(gl.FRAGMENT_SHADER,'out vec4 color;void main(){color=vec4(0.0,1.0,0.0,1.0); }');\
         let p=gl.createProgram();gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);gl.useProgram(p);\
         let vb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,vb);\
         gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,0,1]),gl.STATIC_DRAW);\
         let a=gl.getAttribLocation(p,'p');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,0,0);\
         let eb=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,eb);\
         gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint32Array([0,1,2]),gl.STATIC_DRAW);\
         gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_INT,0);let pixel=new Uint8Array(4);\
         gl.readPixels(2,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);[pixel.join(','),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("0,255,0,255|0".into()));
}
