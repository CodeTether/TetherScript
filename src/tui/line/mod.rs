//! Line shaping and item rendering for terminal frames.

mod fit;
mod item;

pub(super) use fit::fit;
pub(super) use fit::item;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_width;
