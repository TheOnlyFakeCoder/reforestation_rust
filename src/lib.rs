#![allow(clippy::needless_range_loop)]

pub mod algorithm;
pub mod competition;
pub mod formatting;
pub mod map;
pub mod matrix_handler;
pub mod reforestation;
pub mod writer;

pub use reforestation::{Plant, Polygon, Wasted};
