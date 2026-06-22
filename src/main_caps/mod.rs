//! CLI capability grant helpers.

mod browser;
mod fs;
mod interp;
mod provider;
mod vm;

pub(crate) struct RunCaps<'a> {
    pub fs_grant: &'a Option<String>,
    pub full_access: bool,
    pub provider_grant: &'a Option<String>,
    pub provider_key: &'a Option<String>,
    pub provider_vault: &'a Option<String>,
    pub rpc_grant: &'a Option<String>,
    pub browser_grant: &'a Option<String>,
    pub browser_origins: &'a [String],
    pub browser_scopes: &'a [String],
}

pub(crate) use interp::grant as grant_interp;
pub(crate) use vm::grant as grant_vm;
