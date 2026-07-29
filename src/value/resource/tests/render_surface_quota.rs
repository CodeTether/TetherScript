//! Render-surface backpressure quota tests.

use crate::value::resource::OwnedResource;

#[test]
fn surface_rejects_frames_over_its_pixel_quota() {
    let error = OwnedResource::render_surface(8, 4, 2, 127).unwrap_err();
    assert!(error.contains("backpressure"), "{error}");
    assert!(error.contains("128 pixels"), "{error}");
}
