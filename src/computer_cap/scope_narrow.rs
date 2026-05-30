//! Capability attenuation for computer authorities.

use std::rc::Rc;

use crate::capability::Authority;
use crate::value::Value;

use super::authority::ComputerAuthority;

impl ComputerAuthority {
    pub(crate) fn narrowed(&self, params: &Value) -> Result<Rc<dyn Authority>, String> {
        let map = match params {
            Value::Map(m) => m.borrow(),
            _ => return Err("computer.narrow: expected params map".into()),
        };
        let mut next = self.clone();
        if let Some(v) = map.get("scopes") {
            super::scope_apply::scopes(self, &mut next, v)?;
        }
        Ok(Rc::new(next))
    }
}
