use crate::hex_grid::{HexCoord, HexGrid};

#[derive(Clone, Debug)]
pub struct TerrainMap {
    grid: HexGrid,
    cells: Vec<Option<usize>>,
}

impl TerrainMap {
    pub fn new(width: usize, height: usize) -> Self {
        TerrainMap {
            grid: HexGrid::new(width, height),
            cells: vec![None; width * height],
        }
    }

    pub fn from_map(map: &[Vec<i32>]) -> Self {
        let height = map.len();
        let width = map.first().map_or(0, |row| row.len());
        let mut terrain = TerrainMap::new(width, height);

        for (row, values) in map.iter().enumerate() {
            for (col, &value) in values.iter().enumerate() {
                if value != 0 {
                    let coord = HexCoord::from_offset(col as i32, row as i32);
                    terrain.assign(coord, value as usize);
                }
            }
        }

        terrain
    }

    pub fn width(&self) -> i32 {
        self.grid.width
    }

    pub fn height(&self) -> i32 {
        self.grid.height
    }

    fn index(&self, coord: &HexCoord) -> Option<usize> {
        if !self.grid.contains(coord) {
            return None;
        }
        let (col, row) = coord.to_offset();
        Some(row as usize * self.grid.width as usize + col as usize)
    }

    pub fn assign(&mut self, coord: HexCoord, plant_id: usize) -> bool {
        match self.index(&coord) {
            Some(index) => {
                self.cells[index] = Some(plant_id);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self, coord: &HexCoord) -> bool {
        match self.index(coord) {
            Some(index) => {
                let was_occupied = self.cells[index].is_some();
                self.cells[index] = None;
                was_occupied
            }
            None => false,
        }
    }

    pub fn get(&self, coord: &HexCoord) -> Option<usize> {
        self.index(coord).and_then(|index| self.cells[index])
    }

    pub fn is_occupied(&self, coord: &HexCoord) -> bool {
        self.get(coord).is_some()
    }

    pub fn occupied_neighbors(&self, coord: &HexCoord, radius: i32) -> Vec<(HexCoord, usize)> {
        self.grid
            .neighbors_within(coord, radius)
            .into_iter()
            .filter_map(|neighbor| self.get(&neighbor).map(|plant_id| (neighbor, plant_id)))
            .collect()
    }

    pub fn occupied_count(&self) -> usize {
        self.cells.iter().filter(|cell| cell.is_some()).count()
    }

    pub fn iter_occupied(&self) -> impl Iterator<Item = (HexCoord, usize)> + '_ {
        self.cells.iter().enumerate().filter_map(|(index, cell)| {
            cell.map(|plant_id| {
                let width = self.grid.width as usize;
                let row = (index / width) as i32;
                let col = (index % width) as i32;
                (HexCoord::from_offset(col, row), plant_id)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_and_get_round_trip() {
        let mut terrain = TerrainMap::new(5, 5);
        let coord = HexCoord::from_offset(2, 3);
        assert!(terrain.assign(coord, 7));
        assert_eq!(terrain.get(&coord), Some(7));
        assert!(terrain.is_occupied(&coord));
    }

    #[test]
    fn out_of_bounds_operations_are_rejected() {
        let mut terrain = TerrainMap::new(3, 3);
        let outside = HexCoord::from_offset(9, 9);
        assert!(!terrain.assign(outside, 1));
        assert_eq!(terrain.get(&outside), None);
        assert!(!terrain.clear(&outside));
    }

    #[test]
    fn occupied_neighbors_only_returns_filled_cells() {
        let mut terrain = TerrainMap::new(6, 6);
        let center = HexCoord::from_offset(2, 2);
        terrain.assign(center, 1);
        for neighbor in center.neighbors() {
            terrain.assign(neighbor, 2);
        }

        let found = terrain.occupied_neighbors(&center, 1);
        assert_eq!(found.len(), 6);
        assert!(found.iter().all(|(_, id)| *id == 2));
    }

    #[test]
    fn from_map_assigns_non_zero_values() {
        let map = vec![vec![0, 1, 0], vec![2, 0, 3]];
        let terrain = TerrainMap::from_map(&map);
        assert_eq!(terrain.width(), 3);
        assert_eq!(terrain.height(), 2);
        assert_eq!(terrain.occupied_count(), 3);
        assert_eq!(terrain.get(&HexCoord::from_offset(1, 0)), Some(1));
        assert_eq!(terrain.get(&HexCoord::from_offset(0, 1)), Some(2));
        assert_eq!(terrain.get(&HexCoord::from_offset(2, 1)), Some(3));
    }

    #[test]
    fn iter_occupied_matches_count() {
        let map = vec![vec![1, 0, 1], vec![0, 1, 0]];
        let terrain = TerrainMap::from_map(&map);
        assert_eq!(terrain.iter_occupied().count(), terrain.occupied_count());
    }
}
