use std::collections::HashMap;
use std::time::Instant;

use reforestation::algorithm::Algorithm;
use reforestation::competition::get_competition_matrix;
use reforestation::formatting::to_string_double;
use reforestation::map::initialize_unfeasible_map;
use reforestation::matrix_handler::MatrixHandler;
use reforestation::writer::{read_plants_file, read_polygons_file};
use reforestation::Wasted;

fn main() {
    let start = Instant::now();

    let plants = read_plants_file("bins/plants.bin");
    for plant in &plants {
        println!("{}", plant);
    }

    let mut plants_map: HashMap<String, usize> = HashMap::new();
    for (i, plant) in plants.iter().enumerate() {
        plants_map.insert(plant.get_specie().to_string(), i + 1);
    }
    let _ = plants_map.len();

    let _total_per_hectare = 658;
    let _budget = 12500.0;

    let polygons = read_polygons_file("bins/polygons.bin");
    for polygon in &polygons {
        println!("{}", polygon);
    }

    let _surveillance: Vec<Vec<f64>> = vec![
        vec![100.0, 94.35, 92.65],
        vec![100.0, 95.87, 90.82],
        vec![100.0, 82.45, 74.53],
        vec![100.0, 95.17, 91.38],
        vec![100.0, 92.72, 82.87],
        vec![100.0, 89.86, 81.94],
        vec![100.0, 95.76, 91.98],
        vec![100.0, 88.60, 85.25],
        vec![100.0, 82.14, 81.79],
        vec![100.0, 88.43, 72.63],
    ];

    let waste = Wasted::new(polygons.clone(), plants.clone());
    println!("{}", waste);

    let coefficients_matrix = get_competition_matrix(&plants);
    MatrixHandler::print_matrix_elements(&coefficients_matrix);
    let steady_states = Algorithm::markov_steady_state(&coefficients_matrix);

    println!();
    println!("Steady Values");
    MatrixHandler::print_vector_elements(&steady_states);

    Algorithm::polygons_penalty_print(&polygons, &coefficients_matrix);

    for (i, polygon) in polygons.iter().enumerate() {
        let mut sum = 0.0;
        for quantity in polygon.get_quantity_planted() {
            sum += *quantity as f64;
        }
        println!(
            "sum without coeficients P{}: {}",
            i + 1,
            to_string_double(sum / polygon.get_area_hectare())
        );
    }

    MatrixHandler::print_matrix_elements(&initialize_unfeasible_map(
        polygons[0].get_area_hectare(),
    ));

    let duration = start.elapsed();
    println!("Execution Time in ms {}", duration.as_secs_f64() * 1000.0);
}
