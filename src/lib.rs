#![allow(clippy::needless_range_loop)]

pub mod algorithm;
pub mod competition;
pub mod composicion;
pub mod competencia;
pub mod config;
pub mod datos;
pub mod formatting;
pub mod hex_grid;
pub mod interaction;
pub mod malla;
pub mod map;
pub mod matrix_handler;
pub mod reforestation;
pub mod reportes;
pub mod tabu;
pub mod terrain;
pub mod writer;

pub use reforestation::{Plant, Polygon, Wasted};
pub use terrain::TerrainMap;
