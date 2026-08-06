//! Thread-local storage of installed socket grants.

use std::cell::RefCell;

use super::scope::Scope;
use super::transport::Transport;

#[derive(Default)]
pub(super) struct Grants {
    pub(super) tcp: Option<Vec<Scope>>,
    pub(super) udp: Option<Vec<Scope>>,
}

thread_local! {
    static GRANTS: RefCell<Grants> = RefCell::new(Grants::default());
}

/// Install TCP grant patterns for this thread, replacing any previous grant.
///
/// # Errors
///
/// Returns an error naming the offending pattern when one cannot be parsed.
pub fn grant_tcp(patterns: &[String]) -> Result<(), String> {
    let scopes = parse_all(patterns)?;
    GRANTS.with(|grants| grants.borrow_mut().tcp = Some(scopes));
    Ok(())
}

/// Install UDP grant patterns for this thread, replacing any previous grant.
///
/// # Errors
///
/// Returns an error naming the offending pattern when one cannot be parsed.
pub fn grant_udp(patterns: &[String]) -> Result<(), String> {
    let scopes = parse_all(patterns)?;
    GRANTS.with(|grants| grants.borrow_mut().udp = Some(scopes));
    Ok(())
}

/// Grant unrestricted TCP and UDP access, as `--access-mode full` implies.
pub fn grant_all() {
    let all = vec![Scope::parse("*").expect("`*` is a valid scope")];
    GRANTS.with(|grants| {
        let mut grants = grants.borrow_mut();
        grants.tcp = Some(all.clone());
        grants.udp = Some(all);
    });
}

/// Revoke every socket grant on this thread. Used by tests and REPL resets.
pub fn revoke_all() {
    GRANTS.with(|grants| *grants.borrow_mut() = Grants::default());
}

/// Report whether an installed grant permits `host:port`, or `None` when the
/// transport has no grant at all.
pub(super) fn permits(transport: Transport, host: &str, port: u16) -> Option<bool> {
    GRANTS.with(|grants| {
        let grants = grants.borrow();
        let scopes = match transport {
            Transport::Tcp => grants.tcp.as_ref(),
            Transport::Udp => grants.udp.as_ref(),
        };
        scopes.map(|scopes| scopes.iter().any(|scope| scope.permits(host, port)))
    })
}

fn parse_all(patterns: &[String]) -> Result<Vec<Scope>, String> {
    patterns
        .iter()
        .map(|pattern| Scope::parse(pattern))
        .collect()
}
