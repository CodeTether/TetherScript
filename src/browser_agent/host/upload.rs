//! Native file-input upload host action.

use crate::browser_agent::{ActionReport, Locator};
use crate::value::Value;

use super::super::state::HostState;

#[path = "upload_file.rs"]
mod file;
#[path = "upload_paths.rs"]
mod paths;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<ActionReport, String> {
    let selector = super::super::value::string_field(payload, "selector")?;
    let files = paths::parse(payload)?
        .iter()
        .map(|path| file::read(path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut report = state.page.set_input_files(&Locator::css(selector), files)?;
    report.action = "upload".into();
    Ok(report)
}
