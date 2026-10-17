//! LEGACY: malla cuadrada de round(sqrt(658·A)) celdas, sin distancia en metros. Reemplazada
//! por `malla::Malla` (tresbolillo con d explícita).

pub fn map_dimension(area_hectare: f64, plants_per_hectare: f64) -> usize {
    let cells = (plants_per_hectare.max(0.0) * area_hectare.max(0.0))
        .sqrt()
        .round() as usize;
    cells.max(1)
}

pub fn map_creation(area_hectare: f64, plants_per_hectare: f64) -> Vec<Vec<i32>> {
    let empty_plant = 0;
    let dimension = map_dimension(area_hectare, plants_per_hectare);

    vec![vec![empty_plant; dimension]; dimension]
}

pub fn initialize_unfeasible_map(area_hectare: f64, plants_per_hectare: f64) -> Vec<Vec<i32>> {
    let mut initial_map = map_creation(area_hectare, plants_per_hectare);
    let mut count = 0;

    for row in initial_map.iter_mut() {
        for cell in row.iter_mut() {
            *cell = 1;
            count += 1;
        }
    }

    println!("Total numbers of plants: {}", count);

    initial_map
}
