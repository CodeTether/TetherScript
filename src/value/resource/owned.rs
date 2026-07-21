//! Uniform owned-resource container.

use super::{deadline::Deadline, lifecycle::Lifecycle, payload::Payload, ResourceKind};

/// A move-only host handle with explicit lifecycle and deadline state.
///
/// Language values store this type behind `Rc<RefCell<_>>`; moving the value
/// tombstones its source binding while ordinary reads temporarily alias it.
///
/// # Examples
///
/// ```
/// use tetherscript::value::resource::{OwnedResource, ResourceKind};
///
/// let resource = OwnedResource::response_writer(64)?;
/// assert_eq!(resource.kind(), ResourceKind::ResponseWriter);
/// assert!(!resource.is_closed());
/// # Ok::<(), String>(())
/// ```
pub struct OwnedResource {
    pub(super) kind: ResourceKind,
    pub(super) lifecycle: Lifecycle,
    pub(super) deadline: Deadline,
    pub(super) payload: Option<Payload>,
}

impl OwnedResource {
    pub(super) fn new(payload: Payload) -> Self {
        Self {
            kind: payload.kind(),
            lifecycle: Lifecycle::Open,
            deadline: Deadline::default(),
            payload: Some(payload),
        }
    }

    /// Return this resource's stable dynamic type tag.
    pub fn kind(&self) -> ResourceKind {
        self.kind
    }
}
