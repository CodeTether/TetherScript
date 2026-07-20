//! Texture-coordinate wrapping and texel-neighbor selection.

use super::constants;

pub(super) fn nearest(value: f64, length: usize, wrap: u32) -> usize {
    let coordinate = normalized(value, wrap);
    let index = (coordinate * length as f64).floor() as usize;
    if wrap == constants::REPEAT {
        index % length
    } else {
        index.min(length - 1)
    }
}

pub(super) fn linear(value: f64, length: usize, wrap: u32) -> (usize, usize, f64) {
    let position = normalized(value, wrap) * length as f64 - 0.5;
    let lower = position.floor();
    (
        index(lower as isize, length, wrap),
        index(lower as isize + 1, length, wrap),
        position - lower,
    )
}

fn normalized(value: f64, wrap: u32) -> f64 {
    let value = if value.is_finite() { value } else { 0.0 };
    if wrap == constants::REPEAT {
        value.rem_euclid(1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn index(value: isize, length: usize, wrap: u32) -> usize {
    if wrap == constants::REPEAT {
        value.rem_euclid(length as isize) as usize
    } else {
        value.clamp(0, length as isize - 1) as usize
    }
}
