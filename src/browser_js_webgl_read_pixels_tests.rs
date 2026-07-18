use super::*;

#[test]
fn webgl_read_pixels_writes_rgba_bytes_at_webgl2_offset() {
    let result = eval_with_dom(
        "<canvas id='c' width='2' height='2'></canvas>",
        "let gl=document.getElementById('c').getContext('webgl2');\
         gl.clearColor(0.25,0.5,0.75,1);gl.clear(gl.COLOR_BUFFER_BIT);\
         let pixels=new Uint8Array(9);\
         gl.readPixels(0,0,2,1,gl.RGBA,gl.UNSIGNED_BYTE,pixels,1);pixels.join(',');",
    )
    .unwrap();

    assert_eq!(result.value.display(), "0,64,128,191,255,64,128,191,255");
}

#[test]
fn webgl_read_pixels_names_short_destination() {
    let result = eval_with_dom(
        "<canvas id='c' width='1' height='1'></canvas>",
        "let gl=document.getElementById('c').getContext('webgl');\
         gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(3));",
    );
    let error = match result {
        Ok(_) => panic!("short readPixels destination should fail"),
        Err(error) => error,
    };

    assert!(error.contains("readPixels: destination needs 4 bytes, got 3"));
}
