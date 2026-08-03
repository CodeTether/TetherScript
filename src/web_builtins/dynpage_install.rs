//! Registration of the slug, device, and locale built-ins.
//!
//! Split from the cache-key registrations only so neither file exceeds the
//! 50-line budget. The grouping is still cohesive: these read the *request*, while
//! the others derive a *key*.
//!
//! `slug_parse` and `locale_of` return a `Result`, so even an argument-type error
//! is delivered as `Err` and a script can inspect it with `?`. `slug_valid` and
//! `device_class` answer a plain bool and str, so a wrong argument type there is a
//! hard native error — there is no third value for "you called me wrong".

use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::pure_native;
use super::dynpage_args::str_arg;
use super::dynpage_slug;
use crate::system::result_value as wrap;
use crate::value::{Env, Value};

/// Define `slug_parse`, `slug_valid`, `device_class`, and `locale_of` in `env`.
///
/// # Arguments
///
/// * `env` — Environment receiving the bindings.
///
/// # Returns
///
/// Nothing.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "slug_parse",
        pure_native("slug_parse", Some(1), slug),
        false,
    );
    bindings.define("slug_valid", pure_native("slug_valid", Some(1), ok), false);
    bindings.define(
        "device_class",
        pure_native("device_class", Some(1), device),
        false,
    );
    bindings.define(
        "locale_of",
        pure_native("locale_of", Some(2), locale),
        false,
    );
}

/// `slug_parse(path)` -> `Result` of the normalised slug.
fn slug(args: &[Value]) -> Result<Value, String> {
    Ok(wrap(
        str_arg(&args[0], "slug_parse: path")
            .and_then(|path| dynpage_slug::parse(&path, "slug_parse"))
            .map(|slug| Value::Str(Rc::new(slug))),
    ))
}

/// `slug_valid(slug)` -> bool, with no normalisation applied.
fn ok(args: &[Value]) -> Result<Value, String> {
    let slug = str_arg(&args[0], "slug_valid: slug")?;
    Ok(Value::Bool(dynpage_slug::valid(&slug)))
}

// `device` and `locale` live in `dynpage_negotiate`, which keeps this file within the line
// budget; they are re-exported so the registration list below reads unchanged.
use super::dynpage_negotiate::{device, locale};
