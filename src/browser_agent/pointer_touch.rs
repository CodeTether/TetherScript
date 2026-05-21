//! Deterministic touch action support.

use crate::browser_agent::action::{ActionReport, BoundingBox};
use crate::browser_agent::locator::Locator;
use crate::browser_agent::page::BrowserPage;
use crate::browser_agent::{prepare, retry};
use crate::browser_session::TraceEvent;

impl BrowserPage {
    /// Dispatch a deterministic touchstart/touchend tap sequence.
    pub fn touch_tap(&mut self, locator: &Locator) -> Result<ActionReport, String> {
        self.dispatch_touch_action("touch_tap", locator, false)
    }

    /// Dispatch a deterministic touchstart/touchmove/touchend sequence.
    pub fn touch_sequence(&mut self, locator: &Locator) -> Result<ActionReport, String> {
        self.dispatch_touch_action("touch_sequence", locator, true)
    }

    fn dispatch_touch_action(
        &mut self,
        action: &'static str,
        locator: &Locator,
        include_move: bool,
    ) -> Result<ActionReport, String> {
        let (resolved, checks) =
            retry::run(self, action, locator, |page| prepare::click(page, locator))?;
        self.eval_js(&touch_script(
            &resolved.dom.path,
            resolved.bounds,
            include_move,
        ))?;
        self.session
            .trace
            .push(TraceEvent::new(action, format!("{:?}", locator.kind)));
        Ok(ActionReport::new(
            action,
            format!("{:?}", locator.kind),
            resolved.bounds,
            checks,
        ))
    }
}

fn touch_script(path: &[usize], bounds: BoundingBox, include_move: bool) -> String {
    let move_event = if include_move {
        format!(
            "n.dispatchEvent({{type:'touchmove',touches:t.touches,targetTouches:t.targetTouches,changedTouches:[{{{}}}]}});",
            super::pointer_event_fields::touch(bounds)
        )
    } else {
        String::new()
    };
    format!(
        "let n={};let t={{type:'touchstart',touches:[{{{}}}],\
         targetTouches:[{{{}}}],changedTouches:[{{{}}}]}};\
         n.dispatchEvent(t);{}n.dispatchEvent({{type:'touchend',touches:[],\
         targetTouches:[],changedTouches:t.changedTouches}});",
        crate::browser_agent::keyboard_escape::node(path),
        super::pointer_event_fields::touch(bounds),
        super::pointer_event_fields::touch(bounds),
        super::pointer_event_fields::touch(bounds),
        move_event
    )
}
