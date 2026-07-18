use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn compiled_program_and_float_buffer_draw_real_triangle_pixels() {
    let result = eval_with_dom(
        "<canvas id='gl' width='4' height='4'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');\
         let vs=gl.createShader(gl.VERTEX_SHADER);\
         gl.shaderSource(vs,'attribute vec2 a_position;void main(){gl_Position=vec4(a_position,0.0,1.0); }');gl.compileShader(vs);\
         let fs=gl.createShader(gl.FRAGMENT_SHADER);\
         gl.shaderSource(fs,'precision mediump float;void main(){gl_FragColor=vec4(1.0,0.0,0.0,1.0); }');gl.compileShader(fs);\
         let p=gl.createProgram();gl.attachShader(p,vs);gl.attachShader(p,fs);gl.linkProgram(p);gl.useProgram(p);\
         let b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);\
         gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,0,1]),gl.STATIC_DRAW);\
         let a=gl.getAttribLocation(p,'a_position');gl.enableVertexAttribArray(a);gl.vertexAttribPointer(a,2,gl.FLOAT,false,0,0);\
         gl.drawArrays(gl.TRIANGLES,0,3);let pixel=new Uint8Array(4);gl.readPixels(2,1,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);\
         [gl.getShaderParameter(vs,gl.COMPILE_STATUS),gl.getProgramParameter(p,gl.LINK_STATUS),\
         gl.getParameter(gl.ARRAY_BUFFER_BINDING)===b,gl.getParameter(gl.CURRENT_PROGRAM)===p,\
         gl.getBufferParameter(gl.ARRAY_BUFFER,gl.BUFFER_SIZE),pixel.join(','),gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("true|true|true|true|24|255,0,0,255|0".into())
    );
}

#[test]
fn invalid_shader_and_draw_calls_set_specific_webgl_errors() {
    let result = eval_with_dom(
        "<canvas id='gl' width='2' height='2'></canvas>",
        "let gl=document.getElementById('gl').getContext('webgl');let bad=gl.createShader(gl.VERTEX_SHADER);\
         gl.shaderSource(bad,'void main(){}');gl.compileShader(bad);let status=gl.getShaderParameter(bad,gl.COMPILE_STATUS);\
         let log=gl.getShaderInfoLog(bad);let missing=gl.createShader(1)===null;let enumError=gl.getError();\
         gl.drawArrays(gl.TRIANGLES,0,3);[status,log,missing,enumError,gl.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String(
            "false|ERROR: 0:1: vertex shader must write gl_Position|true|1280|1282".into()
        )
    );
}
