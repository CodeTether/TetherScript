//! Built-in registration for terminal UI helpers.

use std::rc::Rc;

use crate::value::{Env, NativeFn, NativeFunc, Value};

type Native = fn(&[Value]) -> Result<Value, String>;

pub(super) fn install(env: &mut Env) {
    define(env, "tui_size", Some(0), super::size::builtin);
    define(env, "tui_clear", Some(0), super::ansi::clear);
    define(env, "tui_enter", Some(0), super::ansi::enter);
    define(env, "tui_leave", Some(0), super::ansi::leave);
    define(env, "tui_cursor", Some(1), super::ansi::cursor);
    define(env, "tui_alt_screen", Some(1), super::ansi::alt_screen);
    define(env, "tui_move_to", Some(2), super::ansi::move_to);
    define(env, "tui_render", Some(1), super::render::render);
    define(env, "tui_present", Some(1), super::render::present);
    define(env, "tui_read_event", None, super::input::read_event);
    define(env, "tui_read_key", Some(0), super::key_read::read_key);
    define(env, "tui_style_open", Some(1), super::style::open_value);
    define(env, "tui_style_reset", Some(0), super::style::reset_value);
    define(
        env,
        "tui_span_render",
        Some(1),
        super::style_span::render_value,
    );
    define(env, "stdio_read", Some(0), super::stdio_io::read);
    define(env, "stdio_write", Some(1), super::stdio_io::write);
    define(env, "stdio_write_err", Some(1), super::stdio_err::write);
    define(env, "jsonrpc_request", Some(3), super::jsonrpc::request);
    define(env, "jsonrpc_response", Some(2), super::jsonrpc::response);
    define(env, "jsonrpc_notify", Some(2), super::jsonrpc::notify);
    define(env, "jsonrpc_error", None, super::jsonrpc_error::error);
}

fn define(env: &mut Env, name: &str, arity: Option<usize>, func: Native) {
    env.define(
        name,
        Value::Native(Rc::new(NativeFn {
            name: name.into(),
            arity,
            func: NativeFunc::Pure(Box::new(func)),
        })),
        false,
    );
}
