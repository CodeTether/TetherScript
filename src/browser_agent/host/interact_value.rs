//! Runtime values returned for native page actions.

use crate::browser_agent::ActionReport;
use crate::value::Value;

pub(super) fn report(report: ActionReport) -> Value {
    let bounds = super::value::map(vec![
        ("x", Value::Int(report.bounds.x)),
        ("y", Value::Int(report.bounds.y)),
        ("width", Value::Int(report.bounds.width)),
        ("height", Value::Int(report.bounds.height)),
    ]);
    super::value::map(vec![
        ("action", super::value::string(report.action)),
        ("locator", super::value::string(report.locator)),
        ("bounds", bounds),
    ])
}
