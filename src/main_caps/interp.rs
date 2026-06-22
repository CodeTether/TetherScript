//! Interpreter capability grants.

use crate::main_caps::{browser, fs, provider, RunCaps};
use crate::{fs_cap, interp::Interpreter, rpc_cap};

pub(crate) fn grant(interp: &mut Interpreter, caps: &RunCaps<'_>) -> Result<(), String> {
    if let Some(root) = fs::root(caps.fs_grant, caps.full_access)? {
        interp.grant("fs", fs_cap::FsAuthority::new(&root));
    }
    if let Some(auth) = provider::authority(
        caps.provider_grant,
        caps.provider_key,
        caps.provider_vault,
        caps.full_access,
    )? {
        interp.grant("provider", auth);
    }
    if let Some(endpoint) = caps.rpc_grant {
        interp.grant("rpc", rpc_cap::RpcAuthority::new(endpoint));
    }
    if let Some(auth) = browser::authority(
        caps.browser_grant,
        caps.browser_origins,
        caps.browser_scopes,
    ) {
        interp.grant("browser", auth);
    }
    Ok(())
}
