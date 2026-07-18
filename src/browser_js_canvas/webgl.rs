//! WebGL software rendering and state-query modules.

use super::*;

macro_rules! webgl_mod {
    ($path:literal, $name:ident) => {
        #[path = $path]
        mod $name;
    };
}

webgl_mod!("webgl_attrs.rs", webgl_attrs);
webgl_mod!("webgl_attrs_state.rs", webgl_attrs_state);
webgl_mod!("webgl_capability.rs", webgl_capability);
webgl_mod!("webgl_clear.rs", webgl_clear);
webgl_mod!("webgl_clear_color.rs", webgl_clear_color);
webgl_mod!("webgl_clear_region.rs", webgl_clear_region);
webgl_mod!("webgl_color_mask.rs", webgl_color_mask);
webgl_mod!("webgl_constants.rs", webgl_constants);
webgl_mod!("webgl_context.rs", webgl_context);
webgl_mod!("webgl_error.rs", webgl_error);
webgl_mod!("webgl_ext.rs", webgl_ext);
webgl_mod!("webgl_methods.rs", webgl_methods);
webgl_mod!("webgl_named_constants.rs", webgl_named_constants);
webgl_mod!("webgl_param_value.rs", webgl_param_value);
webgl_mod!("webgl_params.rs", webgl_params);
webgl_mod!("webgl_pipeline.rs", webgl_pipeline);
webgl_mod!("webgl_read.rs", webgl_read);
webgl_mod!("webgl_scissor.rs", webgl_scissor);
webgl_mod!("webgl_state.rs", webgl_state);
webgl_mod!("webgl_store.rs", webgl_store);
webgl_mod!("webgl_values.rs", webgl_values);
webgl_mod!("webgl_viewport.rs", webgl_viewport);

pub(super) fn context_object(handle: DomHandle, version: u8) -> JsValue {
    webgl_context::context_object(handle, version)
}

pub(super) fn reset_all() {
    webgl_store::reset_all();
}

#[cfg(test)]
#[path = "webgl_tests.rs"]
mod tests;
