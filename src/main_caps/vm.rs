//! VM capability grants.

use crate::main_caps::{browser, provider, RunCaps};
use crate::{fs_cap, rpc_cap, vm::VM};

pub(crate) fn grant(vm: &mut VM, caps: &RunCaps<'_>) -> Result<(), String> {
    if let Some(root) = caps.fs_grant {
        vm.grant("fs", fs_cap::FsAuthority::new(root));
    }
    if let Some(auth) =
        provider::authority(caps.provider_grant, caps.provider_key, caps.provider_vault)?
    {
        vm.grant("provider", auth);
    }
    if let Some(endpoint) = caps.rpc_grant {
        vm.grant("rpc", rpc_cap::RpcAuthority::new(endpoint));
    }
    if let Some(auth) = browser::authority(
        caps.browser_grant,
        caps.browser_origins,
        caps.browser_scopes,
    ) {
        vm.grant("browser", auth);
    }
    Ok(())
}
