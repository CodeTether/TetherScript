//! Script hot-reload runner loop.

#[path = "run_reload_marker.rs"]
mod marker;

pub(crate) fn execute(
    path: &str,
    vm_mode: bool,
    step_budget: Option<u64>,
    fs_grant: &Option<String>,
    full_access: bool,
    provider_grant: &Option<String>,
    provider_key: &Option<String>,
    provider_vault: &Option<String>,
    rpc_grant: &Option<String>,
    browser_grant: &Option<String>,
    browser_origins: &[String],
    browser_scopes: &[String],
) {
    let hot = crate::main_caps::script_hot_reload(&source(path));
    let mut previous = source(path);
    loop {
        if hot {
            marker::clear(path);
        }
        crate::execute_file(
            path,
            vm_mode,
            step_budget,
            fs_grant,
            full_access,
            provider_grant,
            provider_key,
            provider_vault,
            rpc_grant,
            browser_grant,
            browser_origins,
            browser_scopes,
        );
        if !hot || !marker::take(path) {
            break;
        }
        let current = source(path);
        if current == previous {
            break;
        }
        previous = current;
    }
}

fn source(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}
