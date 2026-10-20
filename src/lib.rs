#![allow(clippy::needless_range_loop)]

pub mod algorithm;
pub mod busqueda_tabu;
pub mod calibracion;
pub mod competencia;
pub mod competition;
pub mod composicion;
pub mod config;
pub mod datos;
pub mod escenarios;
pub mod formatting;
pub mod hex_grid;
pub mod interaction;
pub mod malla;
pub mod map;
pub mod matrix_handler;
pub mod objetivo;
pub mod paso1;
pub mod paso2;
pub mod paso3;
pub mod practica;
pub mod reforestation;
pub mod reportes;
pub mod tabu;
pub mod terrain;
pub mod validacion;
pub mod writer;

pub use reforestation::{Plant, Polygon, Wasted};
pub use terrain::TerrainMap;
