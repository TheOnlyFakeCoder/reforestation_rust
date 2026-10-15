#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

const AXIAL_DIRECTIONS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    pub fn distance(&self, other: &HexCoord) -> i32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
    }

    pub fn neighbors(&self) -> [HexCoord; 6] {
        AXIAL_DIRECTIONS.map(|(dq, dr)| HexCoord::new(self.q + dq, self.r + dr))
    }

    pub fn ring(&self, radius: i32) -> Vec<HexCoord> {
        if radius <= 0 {
            return vec![*self];
        }

        let mut ring = Vec::with_capacity((6 * radius) as usize);
        let mut coord = HexCoord::new(
            self.q + AXIAL_DIRECTIONS[4].0 * radius,
            self.r + AXIAL_DIRECTIONS[4].1 * radius,
        );

        for direction in AXIAL_DIRECTIONS {
            for _ in 0..radius {
                ring.push(coord);
                coord = HexCoord::new(coord.q + direction.0, coord.r + direction.1);
            }
        }

        ring
    }

    pub fn within_radius(&self, radius: i32) -> Vec<HexCoord> {
        if radius <= 0 {
            return vec![*self];
        }

        let mut coords = Vec::new();
        for dq in -radius..=radius {
            let dr_min = (-radius).max(-dq - radius);
            let dr_max = radius.min(-dq + radius);
            for dr in dr_min..=dr_max {
                coords.push(HexCoord::new(self.q + dq, self.r + dr));
            }
        }

        coords
    }

    pub fn to_offset(&self) -> (i32, i32) {
        let col = self.q + (self.r - (self.r & 1)) / 2;
        (col, self.r)
    }

    pub fn from_offset(col: i32, row: i32) -> Self {
        let q = col - (row - (row & 1)) / 2;
        HexCoord::new(q, row)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexGrid {
    pub width: i32,
    pub height: i32,
}

impl HexGrid {
    pub fn new(width: usize, height: usize) -> Self {
        HexGrid {
            width: width as i32,
            height: height as i32,
        }
    }

    pub fn contains(&self, coord: &HexCoord) -> bool {
        let (col, row) = coord.to_offset();
        col >= 0 && col < self.width && row >= 0 && row < self.height
    }

    pub fn neighbors_within(&self, coord: &HexCoord, radius: i32) -> Vec<HexCoord> {
        coord
            .within_radius(radius)
            .into_iter()
            .filter(|candidate| candidate != coord && self.contains(candidate))
            .collect()
    }

    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_symmetric_and_zero_on_self() {
        let a = HexCoord::new(2, -1);
        let b = HexCoord::new(-3, 4);
        assert_eq!(a.distance(&a), 0);
        assert_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn distance_matches_known_axial_values() {
        let origin = HexCoord::new(0, 0);
        assert_eq!(origin.distance(&HexCoord::new(3, 0)), 3);
        assert_eq!(origin.distance(&HexCoord::new(0, 3)), 3);
        assert_eq!(origin.distance(&HexCoord::new(-2, -2)), 4);
    }

    #[test]
    fn neighbors_are_six_at_distance_one() {
        let center = HexCoord::new(1, 1);
        let neighbors = center.neighbors();
        assert_eq!(neighbors.len(), 6);
        for neighbor in neighbors {
            assert_eq!(center.distance(&neighbor), 1);
        }
    }

    #[test]
    fn within_radius_contains_expected_count() {
        let center = HexCoord::new(0, 0);
        assert_eq!(center.within_radius(0).len(), 1);
        assert_eq!(center.within_radius(1).len(), 7);
        assert_eq!(center.within_radius(2).len(), 19);
    }

    #[test]
    fn offset_round_trip_preserves_coordinates() {
        for row in -4..4 {
            for col in -4..4 {
                let coord = HexCoord::from_offset(col, row);
                assert_eq!(coord.to_offset(), (col, row));
            }
        }
    }

    #[test]
    fn grid_filters_out_of_bounds_neighbors() {
        let grid = HexGrid::new(3, 3);
        let corner = HexCoord::from_offset(0, 0);
        let neighbors = grid.neighbors_within(&corner, 1);
        assert!(neighbors.iter().all(|coord| grid.contains(coord)));
        assert_eq!(neighbors.len(), 2);
    }
}
