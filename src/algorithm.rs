use crate::formatting::to_string_double;
use crate::reforestation::Polygon;

pub struct Algorithm;

impl Algorithm {
    pub fn new() -> Self {
        Algorithm
    }

    pub fn stochastic_coefficients(coefficients: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = coefficients.len();

        let mut row_sums = Vec::with_capacity(n);
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                sum += coefficients[i][j];
            }
            row_sums.push(sum);
        }

        let mut probabilities = Vec::with_capacity(n);
        for i in 0..n {
            let mut probabilities_row = Vec::with_capacity(n);
            for j in 0..n {
                let singular_probability = if row_sums[i] > 0.0 {
                    coefficients[i][j] / row_sums[i]
                } else {
                    0.0
                };
                probabilities_row.push(singular_probability);
            }
            probabilities.push(probabilities_row);
        }

        probabilities
    }

    pub fn markov_steady_state(coefficients: &[Vec<f64>]) -> Vec<f64> {
        let probabilities = Self::stochastic_coefficients(coefficients);
        let n = probabilities.len();
        if n == 0 {
            return Vec::new();
        }

        let mut v = vec![1.0 / n as f64; n];
        let mut v_next = vec![0.0; n];

        for _ in 0..10000 {
            for value in v_next.iter_mut() {
                *value = 0.0;
            }

            for i in 0..n {
                for j in 0..n {
                    v_next[j] += v[i] * probabilities[i][j];
                }
            }

            let mut diff = 0.0;
            for i in 0..n {
                diff += (v_next[i] - v[i]).abs();
            }

            v.copy_from_slice(&v_next);

            if diff < 1e-18 {
                break;
            }
        }

        v
    }

    pub fn calculate_polygons_penalty(polygon: &Polygon, coefficients: &[Vec<f64>]) -> f64 {
        let steady_states = Self::markov_steady_state(coefficients);
        let quantity_polygons = polygon.get_quantity_planted();
        let mut polygon_penalty = 0.0;
        for i in 0..quantity_polygons.len() {
            polygon_penalty += quantity_polygons[i] as f64 * steady_states[i];
        }
        polygon_penalty / polygon.get_area_hectare()
    }

    pub fn polygons_penalty_print(polygons: &[Polygon], coefficients: &[Vec<f64>]) {
        for (i, polygon) in polygons.iter().enumerate() {
            let penalty = Self::calculate_polygons_penalty(polygon, coefficients);
            println!("Penalty for P{}: {}", i + 1, to_string_double(penalty));
        }
    }
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm
    }
}
