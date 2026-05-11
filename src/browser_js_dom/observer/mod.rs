//! DOM observer modules: MutationObserver, IntersectionObserver, ResizeObserver.

pub mod mutation_types;
pub mod mutation_observer;
pub mod mutation_delivery;
pub mod intersection_types;
pub mod intersection_compute;
pub mod intersection_observer;
pub mod resize_types;
pub mod resize_observer;

#[cfg(test)]
mod mutation_tests;
