//! Language factory for lifecycle-managed rendering surfaces.

use crate::value::Value;

use super::{args, factory, OwnedResource};

pub(super) fn surface(values: &[Value]) -> Result<Value, String> {
    let width = args::usize(&values[0], "resource.render_surface width");
    let height = args::usize(&values[1], "resource.render_surface height");
    let scale = args::usize(&values[2], "resource.render_surface scale");
    let capacity = args::usize(&values[3], "resource.render_surface capacity");
    Ok(factory::resource(width.and_then(|width| {
        height.and_then(|height| {
            scale.and_then(|scale| {
                capacity.and_then(|capacity| {
                    OwnedResource::render_surface(width, height, scale, capacity)
                })
            })
        })
    })))
}
