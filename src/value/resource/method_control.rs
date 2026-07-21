//! Language methods for resource lifecycle controls.

use std::time::Duration;

use crate::value::Value;

use super::{args, result, OwnedResource};

impl OwnedResource {
    pub(super) fn control_call(
        &mut self,
        name: &str,
        values: &[Value],
    ) -> Option<Result<Value, String>> {
        let value = match (name, values) {
            ("kind", []) => Value::Str(std::rc::Rc::new(self.kind.type_name().into())),
            ("close", []) => {
                self.close();
                result::nil(Ok(()))
            }
            ("cancel", []) => result::nil(self.cancel()),
            ("is_closed", []) => Value::Bool(self.is_closed()),
            ("is_cancelled", []) => Value::Bool(self.is_cancelled()),
            ("is_expired", []) => Value::Bool(self.is_expired()),
            ("clear_deadline", []) => {
                self.clear_deadline();
                result::nil(Ok(()))
            }
            ("set_deadline", [delay]) => result::nil(
                args::u64(delay, "resource.set_deadline delay").map(|milliseconds| {
                    self.set_deadline_after(Duration::from_millis(milliseconds));
                }),
            ),
            ("deadline_remaining_ms", []) => self
                .deadline
                .remaining_ms()
                .map_or(Value::Nil, |milliseconds| Value::Int(milliseconds as i64)),
            _ => return None,
        };
        Some(Ok(value))
    }
}
