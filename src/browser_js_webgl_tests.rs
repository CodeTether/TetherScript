use super::*;

#[test]
fn webgl_context_records_metadata_commands_and_parameters() {
    let result = eval_with_dom(
        "<canvas id='c' width='8' height='4'></canvas>",
        "let gl=document.getElementById('c').getContext('webgl'); \
         gl.viewport(1,2,3,4); gl.clearColor(0.25,0.5,0.75,1); gl.clear(gl.COLOR_BUFFER_BIT); \
         gl.getParameter(gl.VENDOR)+':'+gl.getParameter(gl.RENDERER)+':'+\
         gl.getParameter(gl.VIEWPORT).join(',')+':'+gl.getParameter(gl.MAX_TEXTURE_SIZE)+':'+\
         gl.getParameter(gl.SHADING_LANGUAGE_VERSION)+':'+gl.getParameter(gl.MAX_VERTEX_ATTRIBS)+':'+\
         gl.getSupportedExtensions().join('|');",
    )
    .unwrap();
    assert_eq!(
        result.value.display(),
        "tetherscript:tetherscript software rasterizer:1,2,3,4:0:WebGL GLSL ES 1.0 (tetherscript):16:"
    );
    let attrs = match &result.document.children[0] {
        Node::Element(el) => &el.attrs,
        _ => panic!("expected canvas"),
    };
    assert_eq!(
        attrs.get("data-agent-webgl-commands").map(String::as_str),
        Some("viewport|1|2|3|4;clearColor|0.25|0.5|0.75|1;clear|16384")
    );
    let checksum = attrs["data-agent-canvas-checksum"].parse::<u64>().unwrap();
    let pixel = u32::from_be_bytes([64, 128, 191, 255]) as u64;
    assert_eq!(checksum, (1_u64..=32).sum::<u64>() * (pixel + 1));
}
