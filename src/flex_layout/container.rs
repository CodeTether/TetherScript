//! Flex container style data.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignContent {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gap {
    pub row_gap: f32,
    pub column_gap: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexContainer {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub gap: Gap,
}

impl FlexContainer {
    pub fn is_row(&self) -> bool {
        matches!(self.direction, FlexDirection::Row | FlexDirection::RowReverse)
    }

    pub fn is_reverse(&self) -> bool {
        matches!(
            self.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        )
    }
}

impl Default for FlexContainer {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            gap: Gap::default(),
        }
    }
}
