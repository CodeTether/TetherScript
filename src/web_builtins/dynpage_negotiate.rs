//! Device-class and locale negotiation handlers.
//!
//! Split from `dynpage_install` so that file stays within the line budget: registration is
//! one concern, and request-attribute negotiation is another.

use std::rc::Rc;

use super::dynpage_args::str_list_arg;
use super::dynpage_request::headers_of;
use super::{dynpage_device, dynpage_locale};
use crate::system::result_value as wrap;
use crate::value::Value;

/// `device_class(request)` -> `mobile`, `tablet`, or `desktop`.
///
/// # Errors
///
/// Returns an error when the argument is not a request map.
pub(super) fn device(args: &[Value]) -> Result<Value, String> {
    let headers = headers_of(&args[0], "device_class")?;
    Ok(Value::Str(Rc::new(
        dynpage_device::classify(&headers).to_string(),
    )))
}

/// `locale_of(request, supported)` -> `Result` of an element of `supported`.
pub(super) fn locale(args: &[Value]) -> Result<Value, String> {
    Ok(wrap(negotiate(args)))
}

/// The fallible half of `locale_of`, kept separate so both errors are wrapped.
///
/// # Errors
///
/// Returns an error when the request is not a map or `supported` is not a list of str.
fn negotiate(args: &[Value]) -> Result<Value, String> {
    let headers = headers_of(&args[0], "locale_of")?;
    let supported = str_list_arg(&args[1], "locale_of: supported")?;
    let chosen = dynpage_locale::negotiate(&headers, &supported);
    Ok(Value::Str(Rc::new(chosen)))
}
