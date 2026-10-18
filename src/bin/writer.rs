//! LEGACY: genera `bins/*.bin` para la UI y `simulation` (nombres sin corregir). Los datos del
//! Entregable 2 se exportan con `reto datos` a `data/*.csv`.

#![allow(clippy::approx_constant)]

use reforestation::algorithm::Algorithm;
use reforestation::competition::get_competition_matrix;
use reforestation::writer::{write_steady_values, write_to_file_plants, write_to_file_polygons};
use reforestation::{Plant, Polygon};

fn main() {
    let plants = vec![
        Plant::new("Agave lechuguilla", 26.0, 42, [30, 50], [10, 40], 1, -0.4),
        Plant::new("Agave salmiana", 26.0, 196, [30, 50], [20, 60], 2, -0.5),
        Plant::new("Agave scabra", 26.0, 42, [15, 25], [10, 40], 1, -0.2),
        Plant::new("Agave striata", 26.0, 42, [20, 30], [10, 35], 1, -0.3),
        Plant::new(
            "Opuntia cantabrigiensis",
            18.0,
            49,
            [20, 30],
            [5, 25],
            2,
            0.0,
        ),
        Plant::new("Opuntia engelmani", 21.0, 38, [20, 30], [5, 30], 2, 0.1),
        Plant::new("Opuntia robusta", 18.0, 73, [20, 30], [10, 35], 2, 0.2),
        Plant::new("Opuntia streptacanta", 18.0, 64, [20, 35], [10, 35], 2, 0.2),
        Plant::new("Prosopis laevigata", 26.5, 86, [25, 30], [30, 200], 3, 1.0),
        Plant::new("Yucca filifera", 26.0, 26, [20, 30], [20, 100], 2, 0.4),
    ];
    write_to_file_plants(&plants, "bins/plants.bin");

    let plant_matrix: Vec<Vec<i32>> = vec![
        vec![
            8, 58, 66, 10, 65, 67, 41, 32, 58, 47, 48, 36, 50, 35, 40, 70, 56, 45, 56, 69, 54, 75,
            44, 10, 44, 72, 74, 67, 60, 34,
        ],
        vec![
            46, 263, 236, 52, 280, 306, 209, 252, 269, 209, 233, 182, 189, 186, 311, 290, 233, 204,
            223, 285, 287, 292, 288, 59, 155, 291, 342, 309, 280, 195,
        ],
        vec![
            16, 47, 51, 15, 66, 63, 43, 46, 58, 47, 41, 43, 49, 38, 52, 55, 39, 34, 51, 57, 70, 60,
            69, 15, 28, 73, 52, 67, 61, 56,
        ],
        vec![
            16, 49, 50, 9, 61, 61, 49, 40, 53, 41, 37, 44, 28, 43, 62, 54, 46, 33, 48, 75, 62, 68,
            51, 11, 31, 50, 55, 66, 53, 58,
        ],
        vec![
            11, 60, 71, 15, 92, 81, 48, 73, 57, 66, 48, 51, 43, 39, 68, 84, 71, 56, 50, 77, 89,
            104, 69, 17, 39, 66, 70, 86, 76, 55,
        ],
        vec![
            14, 44, 56, 10, 54, 62, 52, 39, 58, 43, 40, 41, 33, 44, 63, 50, 45, 46, 48, 74, 58, 62,
            60, 18, 39, 62, 76, 54, 68, 46,
        ],
        vec![
            18, 111, 100, 18, 124, 118, 87, 86, 94, 96, 94, 68, 73, 73, 132, 105, 83, 82, 93, 107,
            126, 109, 122, 17, 65, 125, 100, 120, 109, 91,
        ],
        vec![
            12, 95, 78, 20, 114, 91, 60, 79, 91, 71, 65, 72, 76, 61, 89, 77, 81, 48, 90, 103, 109,
            95, 100, 10, 50, 101, 100, 103, 85, 63,
        ],
        vec![
            15, 98, 123, 23, 106, 133, 97, 91, 117, 108, 94, 67, 97, 74, 149, 121, 97, 95, 120,
            104, 139, 125, 106, 39, 67, 122, 129, 141, 135, 86,
        ],
        vec![
            9, 39, 28, 11, 41, 45, 27, 26, 37, 29, 27, 19, 25, 23, 53, 40, 32, 17, 33, 25, 44, 43,
            39, 8, 25, 31, 44, 33, 36, 23,
        ],
    ];

    let areas = vec![
        1.28, 6.64, 6.76, 1.38, 8.0, 7.82, 5.53, 5.64, 7.11, 6.11, 5.64, 4.92, 5.05, 4.75, 7.97,
        7.34, 5.98, 5.4, 6.28, 7.6, 8.0, 8.0, 7.67, 1.47, 4.19, 7.52, 8.0, 8.0, 7.56, 5.4,
    ];

    let mut polygons = Vec::with_capacity(areas.len());
    for (i, &area) in areas.iter().enumerate() {
        let column: Vec<i32> = plant_matrix.iter().map(|row| row[i]).collect();
        let polygon_id = format!("P{}", i + 1);
        polygons.push(Polygon::new(&polygon_id, area, column));
    }

    write_to_file_polygons(&polygons, "bins/polygons.bin");

    let coefficients_matrix = get_competition_matrix(&plants);
    let steady_states = Algorithm::markov_steady_state(&coefficients_matrix);
    write_steady_values(&steady_states, "bins/steadyValues.bin");
}
