use super::*;

pub(super) type WindowObject = Rc<RefCell<HashMap<String, JsValue>>>;
type WeakWindowObject = Weak<RefCell<HashMap<String, JsValue>>>;

thread_local! {
    static CURRENT: RefCell<Option<WeakWindowObject>> = const { RefCell::new(None) };
}

pub(super) fn register(window: &JsValue) {
    if let JsValue::Object(window) = window {
        CURRENT.with(|current| *current.borrow_mut() = Some(Rc::downgrade(window)));
    }
}

pub(super) fn current() -> Option<WindowObject> {
    CURRENT.with(|current| current.borrow().as_ref().and_then(Weak::upgrade))
}
