//! TetherScript builtins exposing ad-blocking to scripts.
//!
//! Following the `ratelimit` pattern, the "engine" is ordinary script data —
//! a `List` of rule `Map`s. All functions are pure.

#[path = "adblock_builtins/accessors.rs"]
mod accessors;
#[path = "adblock_builtins/decode.rs"]
mod decode;
#[path = "adblock_builtins/encode.rs"]
mod encode;
#[path = "adblock_builtins/register.rs"]
mod register;

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

/// Install the ad-block builtins into the environment.
pub fn install(env: &Rc<RefCell<Env>>) {
    register::all(env);
}
