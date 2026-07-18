use super::*;

#[derive(Clone)]
pub(super) struct Listener {
    pub callback: JsValue,
    pub capture: bool,
    pub once: bool,
}

pub(super) type Registry = Rc<RefCell<HashMap<String, Vec<Listener>>>>;

pub(super) fn new_registry() -> Registry {
    Rc::new(RefCell::new(HashMap::new()))
}
