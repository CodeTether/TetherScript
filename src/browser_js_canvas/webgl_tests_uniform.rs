use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn uniforms_sub_data_and_resource_deletion_are_live() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         function shader(t,s){let x=gl.createShader(t);gl.shaderSource(x,s);gl.compileShader(x);return x;}\
         let vs=shader(gl.VERTEX_SHADER,'attribute vec2 position;void main(){gl_Position=vec4(position,0.0,1.0); }');\
         let fs=shader(gl.FRAGMENT_SHADER,'precision mediump float;uniform vec4 color;void main(){gl_FragColor=color; }');\
         let p=gl.createProgram();gl.attachShader(p,vs);gl.attachShader(p,fs);gl.linkProgram(p);gl.useProgram(p);\
         let color=gl.getUniformLocation(p,'color');let fake={__webgl_program:p.__webgl_id,__webgl_name:'color'};\
         gl.uniform4f(fake,1,0,0,1);let forged=gl.getError();gl.uniform4f(color,0,1,0,1);\
         let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,16777217,gl.DYNAMIC_DRAW);\
         let oversized=gl.getError();gl.bufferData(gl.ARRAY_BUFFER,24,gl.DYNAMIC_DRAW);\
         gl.bufferSubData(gl.ARRAY_BUFFER,0,new Float32Array([-1,-1,1,-1,0,1]));\
         let a=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,0,0);\
         gl.drawArrays(gl.TRIANGLES,0,3);let pixel=new Uint8Array(4);gl.readPixels(2,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);\
         let live=gl.isBuffer(b);let size=gl.getBufferParameter(gl.ARRAY_BUFFER,gl.BUFFER_SIZE);\
         let usage=gl.getBufferParameter(gl.ARRAY_BUFFER,gl.BUFFER_USAGE);gl.deleteBuffer(b);\
         [pixel.join(','),forged,oversized,size,usage,live,gl.isBuffer(b),gl.getParameter(gl.ARRAY_BUFFER_BINDING)===null,gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("0,255,0,255|1282|1285|24|35048|true|false|true|0".into())
    );
}

#[test]
fn resource_handles_are_identity_checked_and_context_owned() {
    let result = eval_with_dom(
        "<canvas id='a'></canvas><canvas id='b'></canvas>",
        "let a=document.getElementById('a').getContext('webgl');\
         let b=document.getElementById('b').getContext('webgl');\
         let foreign=a.createBuffer();let local=b.createBuffer();\
         b.bindBuffer(b.ARRAY_BUFFER,foreign);let cross=b.getError();\
         let fake={__webgl_kind:'buffer',__webgl_id:local.__webgl_id};\
         b.bindBuffer(b.ARRAY_BUFFER,fake);\
         [cross,b.getError(),b.getParameter(b.ARRAY_BUFFER_BINDING)===null].join('|');",
    )
    .unwrap();

    assert_eq!(result.value, JsValue::String("1282|1282|true".into()));
}
