//! Storage methods implemented through browserctl eval.

use crate::value::Value;

use super::call::BrowserCall;

#[path = "map_storage_indexed.rs"]
mod indexed;
#[path = "map_storage_js.rs"]
mod js;
#[path = "map_storage_read.rs"]
mod read;
#[path = "map_storage_write.rs"]
mod write;

pub(crate) fn prepare(method: &str, args: &[Value]) -> Result<BrowserCall, String> {
    match method {
        "cookies" | "local_storage" | "session_storage" => read::prepare(method, args),
        "indexed_db_summary" => indexed::prepare(args),
        "set_cookie" => write::set_cookie(args),
        "set_local_storage" => write::set_local(args),
        "clear_storage" => write::clear(args),
        _ => unreachable!(),
    }
}
