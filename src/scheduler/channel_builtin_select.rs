//! Selecting over several script channel handles.

use crate::value::Value;

use super::endpoint::Receiver;
use super::{current, handles, result, select, select_args, select_report};

/// Receive from whichever of several channels is ready first.
///
/// # Arguments
///
/// * `values` — `[handles: list of int]`, scanned in list order.
///
/// # Returns
///
/// `Ok(map)` with `status` of `"value"` (plus `index`, `channel`, and `value`),
/// `"end"` (plus `index` and `channel`), or `"parked"`. A task that can only wait
/// on one channel cannot multiplex, so this is what makes channels usable
/// alongside the rest of the async surface.
///
/// # Errors
///
/// Returns `Err` inside the `Result` value when the argument is not a list of
/// known handles, when the list is empty, or when a receiver half was dropped.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
/// use std::rc::Rc;
/// use tetherscript::scheduler::channel::{channel_open, channel_select, channel_send};
/// use tetherscript::value::Value;
///
/// let opened = channel_open(&[Value::Int(1), Value::Str("a".to_string().into())])?;
/// assert!(matches!(opened, Value::Result(_)));
/// channel_send(&[Value::Int(1), Value::Int(3)])?;
/// let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
/// assert!(matches!(channel_select(&[list])?, Value::Result(_)));
/// # Ok::<(), String>(())
/// ```
pub fn channel_select(values: &[Value]) -> Result<Value, String> {
    Ok(result::result(
        select_args::handle_list(&values[0]).and_then(|list| {
            let task = current::current_task();
            let receivers = collect(&list)?;
            let arms: Vec<&Receiver> = receivers.iter().collect();
            let outcome = select::select_recv(&arms, task)?;
            Ok(select_report::describe(outcome, &list))
        }),
    ))
}

/// Resolve each handle to its live receiver half.
fn collect(list: &[i64]) -> Result<Vec<Receiver>, String> {
    list.iter()
        .map(|handle| {
            handles::with(*handle, |(_, receiver)| receiver.clone())?
                .ok_or_else(|| format!("chan_select: receiver {handle} was already dropped"))
        })
        .collect()
}
