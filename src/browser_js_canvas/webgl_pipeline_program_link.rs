//! WebGL program link-state mutation.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "linkProgram".into(),
        native("WebGLRenderingContext.linkProgram", Some(1), move |args| {
            webgl_store::mutate(&handle, version, |state| link(state, args.first()));
            Ok(JsValue::Undefined)
        }),
    );
}

fn link(state: &mut WebGlState, value: Option<&JsValue>) {
    let Some(id) = link_target::id(state, value) else {
        return;
    };
    let result = reflection::reflect(state, id);
    state
        .pipeline
        .uniform_locations
        .retain(|(program, _), _| *program != id);
    let Some(program) = state.pipeline.programs.get_mut(&id) else {
        program::invalid(state);
        return;
    };
    match result {
        Ok((attributes, uniforms, samplers, color)) => {
            program.attributes = attributes;
            program.uniforms = uniforms;
            program.samplers = samplers;
            program.color = color;
            program.linked = true;
            program.log.clear();
        }
        Err(log) => {
            program.linked = false;
            program.log = log;
        }
    }
}
