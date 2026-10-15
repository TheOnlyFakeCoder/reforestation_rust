use std::collections::HashMap;
use std::time::Instant;

use reforestation::algorithm::Algorithm;
use reforestation::competition::get_competition_matrix;
use reforestation::formatting::to_string_double;
use reforestation::hex_grid::HexCoord;
use reforestation::interaction::{
    combine_matrices, competitive_cost_matrix, get_ecological_interaction_matrix,
};
use reforestation::map::initialize_unfeasible_map;
use reforestation::matrix_handler::MatrixHandler;
use reforestation::tabu::{TabuConfig, TabuSearch, DEFAULT_ROW_SEQUENCE};
use reforestation::terrain::TerrainMap;
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

    let plants_per_hectare: f64 = plants
        .iter()
        .map(|plant| plant.get_per_hectare() as f64)
        .sum();
    let unfeasible_map =
        initialize_unfeasible_map(polygons[0].get_area_hectare(), plants_per_hectare);
    MatrixHandler::print_matrix_elements(&unfeasible_map);

    let interaction_matrix = get_ecological_interaction_matrix(&plants);
    println!();
    println!("Ecological Interaction Matrix");
    MatrixHandler::print_matrix_elements(&interaction_matrix);

    let combined_matrix = combine_matrices(&coefficients_matrix, &interaction_matrix);
    println!();
    println!("Combined Competition Matrix");
    MatrixHandler::print_matrix_elements(&combined_matrix);

    let cost_matrix = competitive_cost_matrix(&coefficients_matrix, &interaction_matrix);
    println!();
    println!("Competitive Cost Matrix");
    MatrixHandler::print_matrix_elements(&cost_matrix);

    let terrain = TerrainMap::from_map(&unfeasible_map);
    println!();
    println!(
        "TerrainMap {}x{} | occupied slots: {}",
        terrain.width(),
        terrain.height(),
        terrain.occupied_count()
    );

    let sample = HexCoord::from_offset(0, 0);
    for radius in 1..=3 {
        println!(
            "Occupied neighbors within radius {} of origin: {}",
            radius,
            terrain.occupied_neighbors(&sample, radius).len()
        );
    }

    println!();
    println!("Tabu Search Allocation");
    let tabu_config = TabuConfig {
        budget_per_hectare: _budget,
        ..TabuConfig::default()
    };
    let allocations: Vec<_> = polygons
        .iter()
        .map(|polygon| {
            (
                polygon,
                TabuSearch::new(&plants, polygon, tabu_config)
                    .with_row_sequence(DEFAULT_ROW_SEQUENCE.to_vec())
                    .solve(),
            )
        })
        .collect();
    for (polygon, allocation) in &allocations {
        println!("{} -> {}", polygon.get_polygon_id(), allocation);
    }

    if let Some((polygon, allocation)) = allocations.first() {
        println!();
        println!("=== {} detailed allocation ===", polygon.get_polygon_id());
        println!(
            "{:<4}{:<28}{:>7}{:>8}{:>7}{:>8}{:>8}",
            "#", "species", "min", "target", "max", "placed", "$/ha"
        );
        for (index, plant) in plants.iter().enumerate() {
            println!(
                "{:<4}{:<28}{:>7}{:>8}{:>7}{:>8}{:>8.0}",
                index,
                plant.get_specie(),
                allocation.min_counts[index],
                allocation.target[index],
                allocation.max_counts[index],
                allocation.counts[index],
                plant.get_unitary_cost() * plant.get_per_hectare() as f64,
            );
        }
        println!();
        println!("plan {:.4} ha", polygon.get_area_hectare());
        println!("{}", allocation);
        println!();
        println!("Initial pattern (before optimization):");
        print!("{}", allocation.initial_grid_string());
        println!();
        println!("Optimized grid (species index per cell, . = not plantable):");
        print!("{}", allocation.grid_string());
    }

    let duration = start.elapsed();
    println!("Execution Time in ms {}", duration.as_secs_f64() * 1000.0);
}
