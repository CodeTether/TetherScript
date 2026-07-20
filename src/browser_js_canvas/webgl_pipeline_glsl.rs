//! Supported GLSL ES validation and reflection facade.

#[path = "webgl_pipeline_glsl_declarations.rs"]
mod declarations;
#[path = "webgl_pipeline_glsl_fragment.rs"]
mod fragment;
#[path = "webgl_pipeline_glsl_texture.rs"]
mod texture;
#[path = "webgl_pipeline_glsl_validation.rs"]
mod validation;
#[path = "webgl_pipeline_glsl_varying.rs"]
mod varying;

pub(super) use declarations::{attributes, samplers, uniforms};
pub(super) use fragment::color;
pub(super) use validation::validate;
pub(super) use varying::attribute as varying_attribute;
