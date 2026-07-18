//! Aggregate programmable-pipeline state owned by one WebGL context.

use super::*;

#[derive(Clone, Default)]
pub(in crate::browser_js::canvas_host::webgl) struct State {
    pub(super) next_id: u32,
    pub(super) objects: HashMap<u32, JsValue>,
    pub(super) shaders: HashMap<u32, shader_state::Shader>,
    pub(super) programs: HashMap<u32, shader_state::Program>,
    pub(super) buffers: HashMap<u32, buffer_state::Buffer>,
    pub(super) attributes: HashMap<u32, buffer_state::Attribute>,
    pub(super) bound_array_buffer: Option<u32>,
    pub(super) current_program: Option<u32>,
    pub(super) uniform_locations: HashMap<(u32, String), JsValue>,
}

impl State {
    pub(super) fn allocate(&mut self, kind: &str) -> (u32, JsValue) {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let object = resource::new_object(kind, id);
        self.objects.insert(id, object.clone());
        (id, object)
    }

    pub(super) fn object(&self, id: Option<u32>) -> JsValue {
        id.and_then(|id| self.objects.get(&id).cloned())
            .unwrap_or(JsValue::Null)
    }
}
