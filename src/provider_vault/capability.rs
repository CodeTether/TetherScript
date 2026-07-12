//! Credential-safe Vault access exposed to authority scripts.

use std::any::Any;
use std::rc::Rc;

use crate::capability::{Authority, Capability};
use crate::value::{Runtime, Value};

pub(crate) struct VaultAuthority;

impl Authority for VaultAuthority {
    fn narrow(&self, _params: &Value) -> Result<Rc<dyn Authority>, String> {
        Err("vault: narrowing is not supported".into())
    }

    fn invoke(&self, _rt: &mut dyn Runtime, method: &str, args: &[Value]) -> Result<Value, String> {
        match (method, args) {
            ("provider", [Value::Str(id)]) => {
                let authority = super::load(id)?;
                Ok(Value::Capability(Capability::new_root(
                    "provider", authority,
                )))
            }
            ("provider", [_]) => Err("vault.provider: id must be a string".into()),
            ("provider", _) => Err("vault.provider: expected one provider id".into()),
            _ => Err(format!("vault: no method `{method}` (have: provider)")),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) fn authority() -> Rc<dyn Authority> {
    Rc::new(VaultAuthority)
}
