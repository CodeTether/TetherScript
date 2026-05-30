//! Public computer-use authority implementation.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use crate::capability::Authority;
use crate::value::Value;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Desktop automation authority exposed as a capability value.
#[derive(Clone, Debug)]
pub struct ComputerAuthority {
    pub(crate) endpoint: String,
    pub(crate) allowed_scopes: HashSet<String>,
    pub(crate) origin: Option<String>,
    pub(crate) timeout: Duration,
    pub(crate) trace: Rc<RefCell<Vec<Value>>>,
}

impl ComputerAuthority {
    /// Create a root computer authority for a CodeTether computer bridge.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(endpoint: &str, scopes: Vec<String>) -> Rc<dyn Authority> {
        Self::new_origin_bound(endpoint, scopes, None)
    }

    /// Create a computer authority bound to a host-provided origin.
    pub fn new_origin_bound(
        endpoint: &str,
        scopes: Vec<String>,
        origin: Option<String>,
    ) -> Rc<dyn Authority> {
        Rc::new(Self {
            endpoint: endpoint.trim_end_matches('/').into(),
            allowed_scopes: scopes.into_iter().collect(),
            origin,
            timeout: DEFAULT_TIMEOUT,
            trace: Rc::new(RefCell::new(Vec::new())),
        })
    }

    /// Return every computer scope understood by the authority.
    pub fn all_scopes() -> Vec<String> {
        super::scope::all_scopes()
    }
}
