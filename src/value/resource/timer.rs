//! Owned monotonic timers.

use std::time::{Duration, Instant};

use crate::value::Value;

use super::{args, result};

pub(super) struct Handle {
    ready_at: Instant,
}

impl Handle {
    pub(super) fn after(duration: Duration) -> Self {
        Self {
            ready_at: Instant::now()
                .checked_add(duration)
                .unwrap_or_else(Instant::now),
        }
    }

    pub(super) fn call(&mut self, name: &str, values: &[Value]) -> Result<Value, String> {
        match (name, values) {
            ("ready", []) => Ok(Value::Bool(Instant::now() >= self.ready_at)),
            ("remaining_ms", []) => Ok(Value::Int(self.remaining_ms() as i64)),
            ("reset", [delay]) => Ok(result::nil(self.reset(delay))),
            _ => Err(format!(
                "timer: no method `{name}` accepting {} arguments",
                values.len()
            )),
        }
    }

    fn remaining_ms(&self) -> u64 {
        self.ready_at
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u64::MAX as u128) as u64
    }

    fn reset(&mut self, value: &Value) -> Result<(), String> {
        let milliseconds = args::u64(value, "timer.reset delay")?;
        self.ready_at = Instant::now()
            .checked_add(Duration::from_millis(milliseconds))
            .ok_or_else(|| "timer.reset: delay exceeds monotonic clock range".to_string())?;
        Ok(())
    }
}
