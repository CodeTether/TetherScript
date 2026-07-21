//! Render-surface quota and frame lifecycle tests.

use std::rc::Rc;

use crate::value::resource::OwnedResource;
use crate::value::Value;

use super::{error, ok};

fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.into()))
}

#[test]
fn surface_owns_one_bounded_frame() {
    let mut surface = OwnedResource::render_surface(8, 4, 1, 32).unwrap();
    assert_eq!(
        ok(surface
            .call("render", &[text("<div>ok</div>"), text("")])
            .unwrap()),
        Value::Int(32)
    );
    assert_eq!(surface.call("pixel_count", &[]).unwrap(), Value::Int(32));
    let Value::Bytes(pixels) = ok(surface.call("pixels", &[]).unwrap()) else {
        panic!("rendered pixels should be bytes");
    };
    assert_eq!(pixels.borrow().len(), 128);
    assert!(!surface.call("is_window_open", &[]).unwrap().truthy());
    ok(surface.call("clear", &[]).unwrap());
    assert!(error(surface.call("pixels", &[]).unwrap()).contains("no frame"));
}

#[test]
fn surface_requires_a_frame_before_presenting() {
    let mut surface = OwnedResource::render_surface(8, 4, 1, 32).unwrap();
    let error = error(surface.call("present", &[]).unwrap());
    assert!(error.contains("no frame has been rendered"), "{error}");
    ok(surface.call("close_window", &[]).unwrap());
}

#[cfg(not(feature = "native-window"))]
#[test]
fn surface_explains_how_to_enable_native_windows() {
    let mut surface = OwnedResource::render_surface(8, 4, 1, 32).unwrap();
    let error = error(surface.call("open_window", &[text("test")]).unwrap());
    assert!(error.contains("`native-window` feature"), "{error}");
    let error = error(surface.call("poll_input", &[]).unwrap());
    assert!(error.contains("no native window is open"), "{error}");
}

#[test]
fn surface_rejects_frames_over_its_pixel_quota() {
    let error = OwnedResource::render_surface(8, 4, 2, 127).unwrap_err();
    assert!(error.contains("backpressure"), "{error}");
    assert!(error.contains("128 pixels"), "{error}");
}
