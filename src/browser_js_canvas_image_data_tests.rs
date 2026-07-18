use super::*;

#[test]
fn canvas_image_data_round_trips_clamped_pixels_with_clipping() {
    let result = eval_with_dom(
        "<canvas id='c' width='2' height='2'></canvas>",
        "let ctx=document.getElementById('c').getContext('2d');\
         let image=ctx.createImageData(2,1);let typed=image.data instanceof Uint8ClampedArray;\
         image.data[0]=300;image.data[1]=-5;image.data[2]=127.5;image.data[3]=254.6;\
         let stored=image.data.slice(0,4).join(',');ctx.putImageData(image,1,1);\
         let out=ctx.getImageData(0,0,2,2);typed+':'+(image instanceof ImageData)+':'+\
         (out.data instanceof Uint8ClampedArray)+':'+stored+':'+out.data.join(',');",
    )
    .unwrap();

    assert_eq!(
        result.value.display(),
        "true:true:true:255,0,128,255:0,0,0,0,0,0,0,0,0,0,0,0,255,0,128,255"
    );
}

#[test]
fn canvas_put_image_data_honors_the_dirty_source_rectangle() {
    let result = eval_with_dom(
        "<canvas id='c' width='2' height='1'></canvas>",
        "let ctx=document.getElementById('c').getContext('2d');\
         let data=new Uint8ClampedArray([255,0,0,255,0,255,0,255]);\
         let image=new ImageData(data,2);ctx.putImageData(image,-1,0,1,0,1,1);\
         ctx.getImageData(0,0,2,1).data.join(',');",
    )
    .unwrap();

    assert_eq!(result.value.display(), "0,255,0,255,0,0,0,0");
}
