//! Content-type modifiers parsed from `$` options.

/// Resource type flags from filter-list `$type` modifiers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceType {
    pub script: bool,
    pub image: bool,
    pub stylesheet: bool,
    pub subdocument: bool,
    pub object: bool,
    pub xhr: bool,
    pub other: bool,
    pub websocket: bool,
}

impl ResourceType {
    /// All types allowed (no `$type` modifier present).
    pub fn all() -> Self {
        Self {
            script: true,
            image: true,
            stylesheet: true,
            subdocument: true,
            object: true,
            xhr: true,
            other: true,
            websocket: true,
        }
    }

    /// True when every resource type is allowed.
    pub fn is_all(&self) -> bool {
        self.script
            && self.image
            && self.stylesheet
            && self.subdocument
            && self.object
            && self.xhr
            && self.other
            && self.websocket
    }
}
