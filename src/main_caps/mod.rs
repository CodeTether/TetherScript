//! CLI capability grant helpers.

mod browser;
mod db;
mod db_port;
mod db_sslmode;
mod fs;
mod interp;
mod provider;
mod script;
mod vm;

pub(crate) struct RunCaps<'a> {
    pub fs_grant: &'a Option<String>,
    pub full_access: bool,
    pub db_grant: &'a Option<String>,
    pub provider_grant: &'a Option<String>,
    pub provider_key: &'a Option<String>,
    pub provider_vault: &'a Option<String>,
    pub rpc_grant: &'a Option<String>,
    pub browser_grant: &'a Option<String>,
    pub browser_origins: &'a [String],
    pub browser_scopes: &'a [String],
}

pub(crate) use interp::grant as grant_interp;
pub(crate) use script::full_access as script_full_access;
pub(crate) use script::hot_reload as script_hot_reload;
pub(crate) use vm::grant as grant_vm;

#[cfg(test)]
mod script_tests;

#[cfg(test)]
mod db_tests;

#[cfg(test)]
mod db_reject_tests;

#[cfg(test)]
mod db_sslmode_tests;
