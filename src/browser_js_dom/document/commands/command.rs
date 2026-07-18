use super::*;

#[derive(Clone, Copy)]
pub(super) enum Command {
    Copy,
    Cut,
    Paste,
    Delete,
    InsertText,
    SelectAll,
}

impl Command {
    pub(super) fn parse(value: Option<&JsValue>) -> Option<Self> {
        match value?.display().trim().to_ascii_lowercase().as_str() {
            "copy" => Some(Self::Copy),
            "cut" => Some(Self::Cut),
            "paste" => Some(Self::Paste),
            "delete" => Some(Self::Delete),
            "inserttext" => Some(Self::InsertText),
            "selectall" => Some(Self::SelectAll),
            _ => None,
        }
    }
}
