pub fn map_creation(area_hectare: f64) -> Vec<Vec<i32>> {
    let conversion = 10000;
    let map_squared = (conversion as f64 * area_hectare) as i32;
    let map_dimension = ((map_squared as f64).sqrt() / 1.7).round() as i32;
    let empty_plant = 0;
    let scaler = 1.0;
    let dimension = (map_dimension as f64 / scaler).round() as usize;

    vec![vec![empty_plant; dimension]; dimension]
}

pub fn initialize_unfeasible_map(area_hectare: f64) -> Vec<Vec<i32>> {
    let mut initial_map = map_creation(area_hectare);
    let dimension = initial_map.len();
    let mut count = 0;

    for i in 0..dimension {
        if i % 2 == 0 {
            let mut j = 0;
            while j < dimension {
                count += 1;
                initial_map[i][j] = 1;
                j += 2;
            }
        } else {
            let mut j = 1;
            while j < dimension {
                count += 1;
                initial_map[i][j] = 1;
                j += 2;
            }
        }
    }

    println!("Total numbers of plants: {}", count);

    initial_map
}
