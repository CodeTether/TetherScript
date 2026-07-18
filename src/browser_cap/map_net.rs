//! Network-related browserctl action mappings.

use crate::value::Value;

use super::authority::BrowserAuthority;
use super::call::BrowserCall;

#[path = "map_net_requests.rs"]
mod requests;
#[path = "map_net_wait.rs"]
mod waits;

pub(crate) fn prepare(
    auth: &BrowserAuthority,
    method: &str,
    args: &[Value],
) -> Result<BrowserCall, String> {
    match method {
        "network_log" | "failed_requests" => network_log(method, args),
        "fetch" | "axios" | "xhr" => requests::request(auth, method, args),
        "replay" | "replay_request" => requests::replay(method, args),
        "diagnose" => no_arg_action("diagnose", args),
        "wait_for_request" | "wait_for_response" => waits::prepare(method, args),
        _ => unreachable!(),
    }
}

fn network_log(method: &str, args: &[Value]) -> Result<BrowserCall, String> {
    let mut entries = Vec::new();
    if method == "failed_requests" {
        entries.push(("failed_only", Value::Bool(true)));
    }
    if let Some(v) = args.first() {
        entries.push(("url_contains", v.clone()));
    }
    Ok(call("network_log", entries))
}

fn no_arg_action(action: &str, args: &[Value]) -> Result<BrowserCall, String> {
    super::args::no_args(action, args)?;
    Ok(call(action, Vec::new()))
}

pub(crate) fn call(action: &str, mut entries: Vec<(&str, Value)>) -> BrowserCall {
    entries.insert(0, ("action", super::value::str_value(action)));
    BrowserCall::new(
        action,
        super::actions::scope_for_action(action).unwrap(),
        super::value::map_value(entries),
    )
}
