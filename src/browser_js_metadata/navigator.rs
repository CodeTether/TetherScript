//! Navigator identity metadata and beacon registration.

use std::{collections::HashMap, rc::Rc};

use crate::js::JsValue;

use super::super::{native, SharedBrowserJsRouteHandler};
use super::{beacon, set_str};

#[path = "navigator/battery.rs"]
mod battery;
#[path = "navigator/capabilities.rs"]
mod capabilities;
#[path = "navigator/connection.rs"]
mod connection;
#[path = "navigator/extras.rs"]
mod extras;
#[path = "navigator/identity.rs"]
mod identity;
#[path = "navigator/locks.rs"]
mod locks;
#[path = "navigator/rejection.rs"]
mod rejection;
#[path = "navigator/scheduling.rs"]
mod scheduling;
#[path = "navigator/share.rs"]
mod share;
#[path = "navigator/storage.rs"]
mod storage;
#[path = "navigator/thenable.rs"]
mod thenable;
#[path = "navigator/user_agent_data.rs"]
mod user_agent_data;
#[path = "navigator/vibration.rs"]
mod vibration;

pub(super) fn install(navigator: &JsValue, route_handler: SharedBrowserJsRouteHandler) {
    let JsValue::Object(navigator_object) = navigator else {
        return;
    };
    let shared = Rc::clone(navigator_object);
    let mut navigator = navigator_object.borrow_mut();
    identity::install(&mut navigator);
    battery::install(&mut navigator);
    capabilities::install(&mut navigator);
    extras::install(&mut navigator);
    connection::install(&mut navigator);
    scheduling::install(&mut navigator);
    user_agent_data::install(&mut navigator);
    storage::install(&mut navigator);
    locks::install(&mut navigator);
    share::install(&mut navigator, Rc::clone(&shared));
    vibration::install(&mut navigator, shared);
    beacon::install(&mut navigator, route_handler);
}
