//! CanvasRenderingContext2D object construction.

use super::*;

pub(super) fn context_object(handle: DomHandle) -> JsValue {
    super::store::ensure(&handle);
    let fill_style = Rc::new(RefCell::new("#000000".to_string()));
    let mut obj = HashMap::new();
    obj.insert(
        "fillStyle".into(),
        JsValue::String(fill_style.borrow().clone()),
    );
    install_fill_style(&mut obj, fill_style.clone());
    super::context_rect::install(&mut obj, handle.clone(), fill_style);
    super::image::install(&mut obj, handle);
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn install_fill_style(obj: &mut HashMap<String, JsValue>, fill_style: Rc<RefCell<String>>) {
    obj.insert(
        "__set:fillStyle".into(),
        native(
            "CanvasRenderingContext2D.set_fillStyle",
            Some(1),
            move |args| {
                let value = args.first().unwrap_or(&JsValue::Undefined).display();
                *fill_style.borrow_mut() = value.clone();
                Ok(JsValue::String(value))
            },
        ),
    );
}
