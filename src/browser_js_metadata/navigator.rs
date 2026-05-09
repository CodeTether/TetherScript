//! Navigator identity metadata and beacon registration.

use std::collections::HashMap;

use crate::js::JsValue;

use super::super::native;
use super::super::SharedBrowserJsRouteHandler;
use super::{beacon, set_str};

#[path = "navigator/capabilities.rs"]
mod capabilities;
#[path = "navigator/locks.rs"]
mod locks;
#[path = "navigator/storage.rs"]
mod storage;
#[path = "navigator/thenable.rs"]
mod thenable;
#[path = "navigator/user_agent_data.rs"]
mod user_agent_data;

pub(super) fn install(navigator: &JsValue, route_handler: SharedBrowserJsRouteHandler) {
    let JsValue::Object(navigator) = navigator else {
        return;
    };
    let mut navigator = navigator.borrow_mut();
    identity(&mut navigator);
    capabilities::install(&mut navigator);
    user_agent_data::install(&mut navigator);
    storage::install(&mut navigator);
    locks::install(&mut navigator);
    beacon::install(&mut navigator, route_handler);
}

fn identity(navigator: &mut HashMap<String, JsValue>) {
    navigator.insert("webdriver".into(), JsValue::Bool(false));
    navigator.insert("maxTouchPoints".into(), JsValue::Number(0.0));
    set_str(navigator, "vendor", "TetherScript");
    set_str(navigator, "product", "Gecko");
}
