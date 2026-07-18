#[derive(Clone, Copy, Default)]
pub(super) struct Geometry {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub client_left: i64,
    pub client_top: i64,
    pub client_width: i64,
    pub client_height: i64,
    pub scroll_width: i64,
    pub scroll_height: i64,
    pub scrollable_x: bool,
    pub scrollable_y: bool,
}

impl Geometry {
    pub fn max_left(self) -> i64 {
        if self.scrollable_x {
            self.scroll_width.saturating_sub(self.client_width)
        } else {
            0
        }
    }

    pub fn max_top(self) -> i64 {
        if self.scrollable_y {
            self.scroll_height.saturating_sub(self.client_height)
        } else {
            0
        }
    }
}
