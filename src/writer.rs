use std::fs::File;
use std::io::{Read, Write};

use crate::reforestation::{Plant, Polygon};

pub fn write_steady_values(steady: &[f64], filename: &str) {
    match File::create(filename) {
        Err(_) => println!("Error opening file "),
        Ok(mut out_file) => {
            let vector_size = steady.len() as u64;
            let _ = out_file.write_all(&vector_size.to_le_bytes());
            for value in steady {
                let _ = out_file.write_all(&value.to_le_bytes());
            }
        }
    }
}

pub fn write_to_file_plants(plants: &[Plant], filename: &str) {
    match File::create(filename) {
        Err(_) => eprintln!("Error opening file "),
        Ok(mut out_file) => {
            let vector_size = plants.len() as u64;
            let _ = out_file.write_all(&vector_size.to_le_bytes());
            for plant in plants {
                let _ = plant.write_to(&mut out_file);
            }
        }
    }
}

pub fn write_to_file_polygons(polygons: &[Polygon], filename: &str) {
    match File::create(filename) {
        Err(_) => eprintln!("Error opening file "),
        Ok(mut out_file) => {
            let vector_size = polygons.len() as u64;
            let _ = out_file.write_all(&vector_size.to_le_bytes());
            for polygon in polygons {
                let _ = polygon.write_to(&mut out_file);
            }
        }
    }
}

pub fn read_plants_file(filename: &str) -> Vec<Plant> {
    let mut input_file = match File::open(filename) {
        Err(_) => {
            eprintln!("Error opening file ");
            return Vec::new();
        }
        Ok(file) => file,
    };

    let mut size_buffer = [0u8; 8];
    if input_file.read_exact(&mut size_buffer).is_err() {
        return Vec::new();
    }
    let vector_size = u64::from_le_bytes(size_buffer) as usize;

    let mut plants = Vec::with_capacity(vector_size);
    for _ in 0..vector_size {
        match Plant::read_from(&mut input_file) {
            Ok(plant) => plants.push(plant),
            Err(_) => break,
        }
    }

    plants
}

pub fn read_polygons_file(filename: &str) -> Vec<Polygon> {
    let mut input_file = match File::open(filename) {
        Err(_) => {
            eprintln!("Error opening file ");
            return Vec::new();
        }
        Ok(file) => file,
    };

    let mut size_buffer = [0u8; 8];
    if input_file.read_exact(&mut size_buffer).is_err() {
        return Vec::new();
    }
    let vector_size = u64::from_le_bytes(size_buffer) as usize;

    let mut polygons = Vec::with_capacity(vector_size);
    for _ in 0..vector_size {
        match Polygon::read_from(&mut input_file) {
            Ok(polygon) => polygons.push(polygon),
            Err(_) => break,
        }
    }

    polygons
}
