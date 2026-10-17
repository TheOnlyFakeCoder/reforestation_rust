//! LEGACY: usa la densidad de plantación como rasgo, por eso la matriz sale casi identidad
//! (A. salmiana ≈ 0.007 con todas). Reemplazado por `competencia::matriz_w`.

use crate::reforestation::Plant;

pub fn get_competition_matrix(plants: &[Plant]) -> Vec<Vec<f64>> {
    let avg_heights: Vec<f64> = plants
        .iter()
        .map(|plant| (plant.get_height()[1] + plant.get_height()[0]) as f64 / 2.0)
        .collect();

    let heights_diffs: Vec<Vec<f64>> = avg_heights
        .iter()
        .map(|&height_i| {
            avg_heights
                .iter()
                .map(|&height_j| (height_i - height_j).abs())
                .collect()
        })
        .collect();

    let densities: Vec<i32> = plants.iter().map(|plant| plant.get_per_hectare()).collect();

    let densities_diffs: Vec<Vec<i32>> = densities
        .iter()
        .map(|&density_i| {
            densities
                .iter()
                .map(|&density_j| (density_i - density_j).abs())
                .collect()
        })
        .collect();

    let n = densities_diffs.first().map_or(0, |row| row.len());

    let mut coefficients = Vec::with_capacity(n);
    for i in 0..n {
        let mut row_coefficients = Vec::with_capacity(n);
        for j in 0..n {
            let coefficient = 1.0 / (1.0 + densities_diffs[i][j] as f64 + heights_diffs[i][j]);
            row_coefficients.push(coefficient);
        }
        coefficients.push(row_coefficients);
    }

    coefficients
}
