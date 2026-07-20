use crate::browser_js::eval_with_dom;
use crate::js::JsValue;

#[test]
fn textures_reject_foreign_handles_bad_uploads_and_invalid_state() {
    let result = eval_with_dom(
        "<canvas id='a'></canvas><canvas id='b'></canvas>",
        "let a=document.getElementById('a').getContext('webgl');let b=document.getElementById('b').getContext('webgl');\
         let foreign=a.createTexture();b.bindTexture(b.TEXTURE_2D,foreign);let cross=b.getError();\
         let image=b.createTexture();let fake={__webgl_kind:'texture',__webgl_id:image.__webgl_id};\
         b.bindTexture(b.TEXTURE_2D,fake);let forged=b.getError();b.bindTexture(b.TEXTURE_2D,image);\
         b.texImage2D(b.TEXTURE_2D,0,b.RGBA,2,2,0,b.RGBA,b.UNSIGNED_BYTE,new Uint8Array([1,2]));let length=b.getError();\
         b.texImage2D(b.TEXTURE_2D,0,b.RGBA,2,2,0,b.RGBA,b.UNSIGNED_BYTE,null);\
         b.texSubImage2D(b.TEXTURE_2D,0,2,2,1,1,b.RGBA,b.UNSIGNED_BYTE,new Uint8Array(4));let range=b.getError();\
         b.texParameteri(b.TEXTURE_2D,b.TEXTURE_MAG_FILTER,1);let parameter=b.getError();\
         b.activeTexture(b.TEXTURE0+8);let unit=b.getError();b.deleteTexture(image);\
         [cross,forged,length,range,parameter,unit,b.isTexture(image),b.getParameter(b.TEXTURE_BINDING_2D)===null,b.getError()].join('|');",
    )
    .unwrap();

    assert_eq!(
        result.value,
        JsValue::String("1282|1282|1281|1281|1280|1280|false|true|0".into())
    );
}
