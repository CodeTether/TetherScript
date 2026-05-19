//! Anchor navigation default action.

use super::super::*;

type LocationObject = Rc<RefCell<HashMap<String, JsValue>>>;
type LocationRegistry = RefCell<HashMap<String, LocationObject>>;

thread_local! {
    static LOCATIONS: LocationRegistry = RefCell::new(HashMap::new());
}

pub(super) fn reset() {
    LOCATIONS.with(|locations| locations.borrow_mut().clear());
}

pub(super) fn register_location(root: &Rc<RefCell<Node>>, location: LocationObject) {
    LOCATIONS.with(|locations| {
        locations.borrow_mut().insert(root_key(root), location);
    });
}

pub(super) fn navigate(handle: &DomHandle, el: &Element, event: &JsValue) -> Result<bool, String> {
    if event_flag(event, "__agentClick") {
        return Ok(true);
    }
    if el.attrs.contains_key("download") {
        return Ok(true);
    }
    let Some(href) = el.attrs.get("href").map(|value| value.trim()) else {
        return Ok(true);
    };
    if href.is_empty() || href.to_ascii_lowercase().starts_with("javascript:") {
        return Ok(true);
    }
    if let Some(location) = location_for(handle) {
        let current = location_href(&location);
        let next = resolve_url(href, Some(&current));
        set_location_href(&location, &next);
    }
    Ok(true)
}

fn location_for(handle: &DomHandle) -> Option<LocationObject> {
    LOCATIONS.with(|locations| locations.borrow().get(&root_key(&handle.root)).cloned())
}

fn root_key(root: &Rc<RefCell<Node>>) -> String {
    format!("{:p}", Rc::as_ptr(root))
}
