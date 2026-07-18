//! Page-level trusted coordinate click action.

use crate::browser_agent::action_checks::ActionabilityReport;
use crate::browser_agent::{ActionReport, BrowserPage};
use crate::browser_session::TraceEvent;

#[cfg(test)]
#[path = "point_tests.rs"]
mod tests;

impl BrowserPage {
    /// Click the topmost element at viewport coordinates `x` and `y`.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal viewport coordinate.
    /// * `y` - Vertical viewport coordinate.
    ///
    /// # Returns
    ///
    /// Metadata for the trusted pointer action and the element it hit.
    ///
    /// # Errors
    ///
    /// Returns an error when the point hits no element or a disabled element.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::browser_agent::BrowserPage;
    ///
    /// let html = "<button style='width:10px;height:10px'>Go</button>";
    /// let mut page = BrowserPage::from_html("mem://point", html);
    /// let report = page.click_at(1, 1).unwrap();
    /// assert_eq!(report.action, "mouse_click");
    /// ```
    pub fn click_at(&mut self, x: i64, y: i64) -> Result<ActionReport, String> {
        let (resolved, page_x, page_y) = super::point_target::resolve(self, x, y)?;
        super::point_dispatch::run(self, &resolved, x, y, page_x, page_y)?;
        self.session
            .trace
            .push(TraceEvent::new("mouse_click", format!("point({x},{y})")));
        Ok(ActionReport::new(
            "mouse_click",
            format!("point({x},{y})"),
            resolved.bounds,
            ActionabilityReport::new(false),
        ))
    }
}
