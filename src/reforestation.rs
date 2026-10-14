use std::fmt;
use std::io::{self, Read, Write};

use crate::formatting::format_g;

pub const SPECIE_LEN: usize = 32;
pub const POLYGON_ID_LEN: usize = 16;
pub const PLANT_SERIALIZED_SIZE: usize = 56;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Plant {
    pub specie: [u8; SPECIE_LEN],
    pub unitary_cost: f64,
    pub per_hectare: i32,
    pub height: [i32; 2],
    pub root_depth: [i32; 2],
    pub root_spread_radius: i32,
    pub interaction_index: f32
}

impl Plant {
    pub fn new(
        specie: &str, 
        unitary_cost: f64, 
        per_hectare: i32, 
        height: [i32; 2], 
        root_depth: [i32; 2],
        root_spread_radius: i32,
        interaction_index: f32) -> Self {
        Plant {
            specie: Self::encode_id(specie),
            unitary_cost,
            per_hectare,
            height,
            root_depth,
            root_spread_radius,
            interaction_index
        }
    }

    pub fn get_specie(&self) -> &str {
        decode_id(&self.specie)
    }

    pub fn get_unitary_cost(&self) -> f64 {
        self.unitary_cost
    }

    pub fn get_per_hectare(&self) -> i32 {
        self.per_hectare
    }

    pub fn get_height(&self) -> [i32; 2] {
        self.height
    }

    pub fn get_root_depth(&self) -> [i32; 2] {
        self.root_depth
    }

    pub fn get_root_spread_radius(&self) -> i32 {
        self.root_spread_radius
    }

    pub fn get_interaction_index(&self) -> f32 {
        self.interaction_index
    }
    pub fn set_specie(&mut self, specie: &str) {
        self.specie = Self::encode_id(specie);
    }

    pub fn set_unitary_cost(&mut self, unitary_cost: f64) {
        self.unitary_cost = unitary_cost;
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.specie)?;
        writer.write_all(&self.unitary_cost.to_le_bytes())?;
        writer.write_all(&self.per_hectare.to_le_bytes())?;
        writer.write_all(&self.height[0].to_le_bytes())?;
        writer.write_all(&self.height[1].to_le_bytes())?;
        writer.write_all(&self.root_depth[0].to_le_bytes())?;
        writer.write_all(&self.root_depth[1].to_le_bytes())?;
        writer.write_all(&self.root_spread_radius.to_le_bytes())?;
        writer.write_all(&self.interaction_index.to_le_bytes())?;
        writer.write_all(&[0u8; 4])?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut specie = [0u8; SPECIE_LEN];
        reader.read_exact(&mut specie)?;

        let mut f64_buffer = [0u8; 8];
        reader.read_exact(&mut f64_buffer)?;
        let unitary_cost = f64::from_le_bytes(f64_buffer);

        let mut i32_buffer = [0u8; 4];
        reader.read_exact(&mut i32_buffer)?;
        let per_hectare = i32::from_le_bytes(i32_buffer);

        let mut height = [0i32; 2];
        for value in height.iter_mut() {
            reader.read_exact(&mut i32_buffer)?;
            *value = i32::from_le_bytes(i32_buffer);
        }

        let mut root_depth = [0i32; 2];
        for value in root_depth.iter_mut() {
            reader.read_exact(&mut i32_buffer)?;
            *value = i32::from_le_bytes(i32_buffer);
        }

        let mut i32_buffer = [0u8; 4];
        reader.read_exact(&mut i32_buffer)?;
        let root_spread_radius = i32::from_le_bytes(i32_buffer);
        
        let mut f32_buffer= [0u8;4];
        reader.read_exact(&mut f32_buffer)?;
        let interaction_index = f32::from_le_bytes(f32_buffer);

        let mut padding = [0u8; 4];
        reader.read_exact(&mut padding)?;

        Ok(Plant {
            specie,
            unitary_cost,
            per_hectare,
            height,
            root_depth,
            root_spread_radius,
            interaction_index
        })
    }

    fn encode_id(text: &str) -> [u8; SPECIE_LEN] {
        let mut buffer: [u8; 32] = [0u8; SPECIE_LEN];
        let bytes = text.as_bytes();
        let length = bytes.len().min(SPECIE_LEN - 1);
        buffer[..length].copy_from_slice(&bytes[..length]);
        buffer
    }
}

impl Default for Plant {
    fn default() -> Self {
        Plant {
            specie: [0u8; SPECIE_LEN],
            unitary_cost: 0.0,
            per_hectare: 0,
            height: [0, 0],
            root_depth: [0,0],
            root_spread_radius: 0,
            interaction_index: 0.0
        }
    }
}

impl fmt::Display for Plant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Plants Name: {} | Cost: {} | Per Hectare: {} | Height {}-{} | Root Depth {}-{} | Root Spread Radius {} | Facilitation Index {}" ,
            self.get_specie(),
            format_g(self.unitary_cost, 6),
            self.per_hectare,
            self.height[0],
            self.height[1],
            self.root_depth[0],
            self.root_depth[1],
            self.root_spread_radius,
            self.interaction_index
        )
    }
}

#[derive(Clone, Debug)]
pub struct Polygon {
    pub polygon: [u8; POLYGON_ID_LEN],
    pub area_hectare: f64,
    pub quantity_planted: Vec<i32>,
}

impl Polygon {
    pub fn new(polygon: &str, area_hectare: f64, quantity_planted: Vec<i32>) -> Self {
        Polygon {
            polygon: Self::encode_id(polygon),
            area_hectare,
            quantity_planted,
        }
    }

    pub fn from_raw_id(
        polygon: [u8; POLYGON_ID_LEN],
        area_hectare: f64,
        quantity_planted: Vec<i32>,
    ) -> Self {
        Polygon {
            polygon,
            area_hectare,
            quantity_planted,
        }
    }

    pub fn get_polygon_id(&self) -> &str {
        decode_id(&self.polygon)
    }

    pub fn get_area_hectare(&self) -> f64 {
        self.area_hectare
    }

    pub fn get_quantity_planted(&self) -> &[i32] {
        &self.quantity_planted
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.polygon)?;
        writer.write_all(&self.area_hectare.to_le_bytes())?;
        writer.write_all(&(self.quantity_planted.len() as u64).to_le_bytes())?;
        for quantity in &self.quantity_planted {
            writer.write_all(&quantity.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut polygon = [0u8; POLYGON_ID_LEN];
        reader.read_exact(&mut polygon)?;

        let mut f64_buffer = [0u8; 8];
        reader.read_exact(&mut f64_buffer)?;
        let area_hectare = f64::from_le_bytes(f64_buffer);

        let mut size_buffer = [0u8; 8];
        reader.read_exact(&mut size_buffer)?;
        let quantity_size = u64::from_le_bytes(size_buffer) as usize;

        let mut quantity_planted = vec![0i32; quantity_size];
        let mut i32_buffer = [0u8; 4];
        for quantity in quantity_planted.iter_mut() {
            reader.read_exact(&mut i32_buffer)?;
            *quantity = i32::from_le_bytes(i32_buffer);
        }

        Ok(Polygon {
            polygon,
            area_hectare,
            quantity_planted,
        })
    }

    fn encode_id(text: &str) -> [u8; POLYGON_ID_LEN] {
        let mut buffer = [0u8; POLYGON_ID_LEN];
        let bytes = text.as_bytes();
        let length = bytes.len().min(POLYGON_ID_LEN - 1);
        buffer[..length].copy_from_slice(&bytes[..length]);
        buffer
    }
}

impl Default for Polygon {
    fn default() -> Self {
        Polygon {
            polygon: [0u8; POLYGON_ID_LEN],
            area_hectare: 0.0,
            quantity_planted: Vec::new(),
        }
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Polygon {} | Area in Hectare {} | Quantity Planted: [",
            self.get_polygon_id(),
            format_g(self.area_hectare, 6)
        )?;

        for (index, quantity) in self.quantity_planted.iter().enumerate() {
            if index + 1 < self.quantity_planted.len() {
                write!(formatter, "{}, ", quantity)?;
            } else {
                write!(formatter, "{}", quantity)?;
            }
        }

        write!(formatter, "]")
    }
}

pub struct Wasted {
    polygons: Vec<Polygon>,
    plants: Vec<Plant>,
    waste_per_polygon: Vec<Vec<f64>>,
}

impl Wasted {
    pub fn new(polygons: Vec<Polygon>, plants: Vec<Plant>) -> Self {
        let waste_per_polygon = Self::calculate_waste_per_polygon(&polygons, &plants);
        Wasted {
            polygons,
            plants,
            waste_per_polygon,
        }
    }

    pub fn get_polygons(&self) -> &[Polygon] {
        &self.polygons
    }

    pub fn get_plants(&self) -> &[Plant] {
        &self.plants
    }

    pub fn get_waste_per_polygon(&self) -> &[Vec<f64>] {
        &self.waste_per_polygon
    }

    pub fn calculate_waste_per_polygon(polygons: &[Polygon], plants: &[Plant]) -> Vec<Vec<f64>> {
        let mut local_waste_matrix = Vec::with_capacity(polygons.len());

        for polygon in polygons {
            let mut per_polygon = Vec::with_capacity(plants.len());
            let area = polygon.get_area_hectare();
            let quantity = polygon.get_quantity_planted();

            for (index, plant) in plants.iter().enumerate() {
                let ideal_quantity = plant.get_per_hectare() as f64 * area;
                let deficit = (ideal_quantity - quantity[index] as f64) / ideal_quantity;
                per_polygon.push(deficit * 100.0);
            }

            local_waste_matrix.push(per_polygon);
        }

        local_waste_matrix
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Wasted {
    fn default() -> Self {
        Wasted {
            polygons: Vec::new(),
            plants: Vec::new(),
            waste_per_polygon: Vec::new(),
        }
    }
}

impl fmt::Display for Wasted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "=== Waste Analysis Per Polygon ===")?;

        for (i, waste_row) in self.waste_per_polygon.iter().enumerate() {
            if i < self.polygons.len() {
                write!(
                    formatter,
                    "Polygon {} -> [",
                    self.polygons[i].get_polygon_id()
                )?;
            } else {
                write!(formatter, "Polygon {} -> [", i)?;
            }

            for (j, waste) in waste_row.iter().enumerate() {
                write!(formatter, "{}", format_g(*waste, 6))?;
                if j + 1 < waste_row.len() {
                    write!(formatter, ", ")?;
                }
            }
            writeln!(formatter, "]")?;
        }

        Ok(())
    }
}

fn decode_id<const N: usize>(buffer: &[u8; N]) -> &str {
    let length = buffer
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(buffer.len());
    std::str::from_utf8(&buffer[..length]).unwrap_or("")
}
