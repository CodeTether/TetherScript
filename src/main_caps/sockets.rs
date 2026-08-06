//! Installing socket grants from parsed CLI flags.
//!
//! Socket authority is thread-local rather than a `Value::Capability`, so it does
//! not thread through `RunCaps`. Installing it here keeps the already-14-argument
//! `run_reload::execute` signature from growing two more parameters.

/// Install TCP and UDP grants, exiting with status 2 on a malformed pattern.
///
/// `--access-mode full` grants both transports unrestricted access, matching how
/// the other capabilities treat full access.
pub(crate) fn install(tcp: &[String], udp: &[String], full_access: bool) {
    if full_access {
        crate::socket_cap::grant_all();
        return;
    }
    if !tcp.is_empty() {
        fail_on_error(crate::socket_cap::grant_tcp(tcp));
    }
    if !udp.is_empty() {
        fail_on_error(crate::socket_cap::grant_udp(udp));
    }
}

fn fail_on_error(result: Result<(), String>) {
    if let Err(error) = result {
        eprintln!("tetherscript run: {error}");
        std::process::exit(2);
    }
}
