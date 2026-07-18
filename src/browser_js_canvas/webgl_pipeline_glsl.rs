//! Supported GLSL ES validation and reflection facade.

#[path = "webgl_pipeline_glsl_declarations.rs"]
mod declarations;
#[path = "webgl_pipeline_glsl_fragment.rs"]
mod fragment;
#[path = "webgl_pipeline_glsl_validation.rs"]
mod validation;

pub(super) use declarations::{attributes, uniforms};
pub(super) use fragment::color;
pub(super) use validation::validate;
