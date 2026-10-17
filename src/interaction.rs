//! LEGACY: índice de interacción sin fuente ni decaimiento con la distancia. Reemplazado por
//! `competencia` (W de Gower y factor de alcance h).

use crate::reforestation::Plant;

const FACILITATION_FLOOR: f64 = 0.05;

pub fn root_overlap_ratio(a: [i32; 2], b: [i32; 2]) -> f64 {
    let overlap_low = a[0].max(b[0]);
    let overlap_high = a[1].min(b[1]);
    let overlap = (overlap_high - overlap_low).max(0) as f64;

    let union_low = a[0].min(b[0]);
    let union_high = a[1].max(b[1]);
    let union = (union_high - union_low) as f64;

    if union <= 0.0 {
        0.0
    } else {
        (overlap / union).clamp(0.0, 1.0)
    }
}

pub fn ecological_factor(a: &Plant, b: &Plant) -> f64 {
    let overlap = root_overlap_ratio(a.get_root_depth(), b.get_root_depth());
    let interaction = (a.get_interaction_index() as f64 + b.get_interaction_index() as f64) / 2.0;

    if interaction >= 0.0 {
        (1.0 - interaction * overlap).max(FACILITATION_FLOOR)
    } else {
        1.0 + interaction.abs() * overlap
    }
}

pub fn get_ecological_interaction_matrix(plants: &[Plant]) -> Vec<Vec<f64>> {
    let n = plants.len();
    let mut matrix = vec![vec![1.0f64; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i != j {
                matrix[i][j] = ecological_factor(&plants[i], &plants[j]);
            }
        }
    }

    matrix
}

pub fn combine_matrices(competition: &[Vec<f64>], interaction: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = competition.len().min(interaction.len());
    let mut combined = Vec::with_capacity(n);

    for i in 0..n {
        let cols = competition[i].len().min(interaction[i].len());
        let mut row = Vec::with_capacity(cols);
        for j in 0..cols {
            row.push(competition[i][j] * interaction[i][j]);
        }
        combined.push(row);
    }

    combined
}

pub fn competitive_cost_matrix(
    competition: &[Vec<f64>],
    interaction: &[Vec<f64>],
) -> Vec<Vec<f64>> {
    combine_matrices(competition, interaction)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|coefficient| {
                    if coefficient > 0.0 {
                        1.0 / coefficient
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plant(interaction: f32, root_depth: [i32; 2]) -> Plant {
        Plant::new("T", 1.0, 10, [10, 20], root_depth, 1, interaction)
    }

    #[test]
    fn no_overlap_yields_no_interaction() {
        let a = plant(-0.5, [0, 10]);
        let b = plant(0.5, [20, 30]);
        assert_eq!(ecological_factor(&a, &b), 1.0);
    }

    #[test]
    fn overlap_ratio_bounds_and_value() {
        assert_eq!(root_overlap_ratio([0, 10], [5, 20]), 5.0 / 20.0);
        assert_eq!(root_overlap_ratio([0, 10], [10, 20]), 0.0);
        assert_eq!(root_overlap_ratio([0, 10], [0, 10]), 1.0);
    }

    #[test]
    fn positive_index_discounts_competition() {
        let a = plant(1.0, [0, 10]);
        let b = plant(1.0, [0, 10]);
        assert!(ecological_factor(&a, &b) < 1.0);
    }

    #[test]
    fn negative_index_adds_penalty() {
        let a = plant(-1.0, [0, 10]);
        let b = plant(-1.0, [0, 10]);
        assert!(ecological_factor(&a, &b) > 1.0);
    }

    #[test]
    fn matrix_is_symmetric_with_unit_diagonal() {
        let plants = vec![plant(0.4, [0, 30]), plant(-0.6, [10, 40])];
        let matrix = get_ecological_interaction_matrix(&plants);
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0][0], 1.0);
        assert_eq!(matrix[1][1], 1.0);
        assert!((matrix[0][1] - matrix[1][0]).abs() < 1e-12);
    }
}
