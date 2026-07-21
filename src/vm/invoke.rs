//! Execution loop for runtime-initiated calls.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

use super::super::{format_unwind, Unwind, VM};

impl VM {
    pub(super) fn invoke_mode(
        &mut self,
        callee: &Value,
        args: &[Value],
        scheduled: bool,
    ) -> Result<Value, String> {
        let depth = self.frames.len();
        let dispatched = if scheduled {
            self.dispatch_scheduled_call(callee.clone(), args.to_vec())
        } else {
            self.dispatch_call(callee.clone(), args.to_vec())
        };
        if let Err(error) = dispatched {
            return Err(format_unwind(error));
        }
        while self.frames.len() > depth {
            let (instruction, code_len) = {
                let frame = self.frames.last().unwrap();
                let code_len = frame.proto.chunk.code.len();
                if frame.ip >= code_len {
                    self.stack.push(Value::Nil);
                    self.do_return();
                    continue;
                }
                (frame.proto.chunk.code[frame.ip].clone(), code_len)
            };
            self.frames.last_mut().unwrap().ip += 1;
            match self.step(instruction, code_len) {
                Ok(()) => {}
                Err(Unwind::TryErr(error)) => {
                    self.stack
                        .push(Value::Result(Rc::new(ResultValue::Err(error))));
                    self.do_return();
                }
                Err(error) => return Err(format_unwind(error)),
            }
        }
        Ok(self.stack.pop().unwrap_or(Value::Nil))
    }
}
