use super::*;

#[test]
fn canvas_image_data_errors_name_invalid_sizes_and_pixel_storage() {
    let zero = eval_with_dom(
        "<canvas id='c'></canvas>",
        "document.getElementById('c').getContext('2d').createImageData(0,1);",
    );
    let zero = match zero {
        Ok(_) => panic!("zero-width ImageData should fail"),
        Err(error) => error,
    };
    assert!(zero.contains("createImageData: width and height must be non-zero"));

    let untyped = eval_with_dom(
        "<canvas id='c'></canvas>",
        "let ctx=document.getElementById('c').getContext('2d');\
         ctx.putImageData({__image_data:true,width:1,height:1,data:[0,0,0,0]},0,0);",
    );
    let untyped = match untyped {
        Ok(_) => panic!("untyped ImageData storage should fail"),
        Err(error) => error,
    };
    assert!(untyped.contains("putImageData: source data must be Uint8ClampedArray"));
}
