use std::collections::HashMap;
use std::fmt;

use crate::competition::get_competition_matrix;
use crate::hex_grid::HexCoord;
use crate::interaction::{combine_matrices, get_ecological_interaction_matrix};
use crate::map::map_dimension;
use crate::reforestation::{Plant, Polygon};

pub const COUNT_TOLERANCE: f64 = 0.10;

pub const DEFAULT_ROW_SEQUENCE: &[usize] = &[
    1, 8, 7, 6, 2, 4, 1, 5, 3, 1, 5, 8, 1, 4, 0, 1, 7, 9, 1, 6, 8,
];

#[derive(Clone, Copy, Debug)]
pub struct TabuConfig {
    pub max_iterations: usize,
    pub tabu_tenure: usize,
    pub max_no_improve: usize,
    pub candidate_list_size: usize,
    pub seed: u64,
    pub budget_per_hectare: f64,
}

impl Default for TabuConfig {
    fn default() -> Self {
        TabuConfig {
            max_iterations: 10000,
            tabu_tenure: 12,
            max_no_improve: 60,
            candidate_list_size: 256,
            seed: 0x5EED_1234,
            budget_per_hectare: 12_500.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CountBand {
    target: Vec<usize>,
    min: Vec<usize>,
    max: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Allocation {
    pub dimension: usize,
    pub coords: Vec<HexCoord>,
    pub initial_species: Vec<Option<usize>>,
    pub initial_counts: Vec<usize>,
    pub species: Vec<Option<usize>>,
    pub counts: Vec<usize>,
    pub target: Vec<usize>,
    pub min_counts: Vec<usize>,
    pub max_counts: Vec<usize>,
    pub initial_cost: f64,
    pub best_cost: f64,
    pub iterations: usize,
    pub improvements: usize,
    pub tabu_hits: usize,
    pub monetary_cost_total: f64,
    pub monetary_cost_per_hectare: f64,
    pub budget_per_hectare: f64,
    pub within_budget: bool,
}

impl Allocation {
    pub fn placed(&self) -> usize {
        self.counts.iter().sum()
    }

    pub fn meets_requirements(&self) -> bool {
        self.counts
            .iter()
            .enumerate()
            .all(|(i, &count)| count >= self.min_counts[i] && count <= self.max_counts[i])
    }

    pub fn grid_string(&self) -> String {
        self.grid_string_for(&self.species)
    }

    pub fn initial_grid_string(&self) -> String {
        self.grid_string_for(&self.initial_species)
    }

    fn grid_string_for(&self, species: &[Option<usize>]) -> String {
        let mut grid = vec![vec![None::<usize>; self.dimension]; self.dimension];
        for (index, coord) in self.coords.iter().enumerate() {
            let (col, row) = coord.to_offset();
            if (0..self.dimension as i32).contains(&col)
                && (0..self.dimension as i32).contains(&row)
            {
                grid[row as usize][col as usize] = species[index];
            }
        }

        let mut output = String::new();
        for row in &grid {
            for cell in row {
                match cell {
                    Some(species) => output.push_str(&format!("{species:>2} ")),
                    None => output.push_str(" . "),
                }
            }
            while output.ends_with(' ') {
                output.pop();
            }
            output.push('\n');
        }
        output
    }
}

impl fmt::Display for Allocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reduction = if self.initial_cost > 0.0 {
            (self.initial_cost - self.best_cost) / self.initial_cost * 100.0
        } else {
            0.0
        };
        write!(
            formatter,
            "grid {}x{} | target {} band [{}..{}] | placed {} | eco {:.2} -> {:.2} ({:.1}%) | iters {} | improved {} | tabu-hits {} | $/ha {:.0} (budget {:.0}) {}",
            self.dimension,
            self.dimension,
            self.target.iter().sum::<usize>(),
            self.min_counts.iter().sum::<usize>(),
            self.max_counts.iter().sum::<usize>(),
            self.placed(),
            self.initial_cost,
            self.best_cost,
            reduction,
            self.iterations,
            self.improvements,
            self.tabu_hits,
            self.monetary_cost_per_hectare,
            self.budget_per_hectare,
            if self.within_budget { "ok" } else { "OVER" },
        )
    }
}

pub struct TabuSearch<'a> {
    plants: &'a [Plant],
    area_hectare: f64,
    penalty: Vec<Vec<f64>>,
    radii: Vec<i32>,
    coords: Vec<HexCoord>,
    adjacency: Vec<Vec<(usize, i32)>>,
    config: TabuConfig,
    row_sequence: Vec<usize>,
    plants_per_hectare: f64,
}

impl<'a> TabuSearch<'a> {
    pub fn new(plants: &'a [Plant], polygon: &Polygon, config: TabuConfig) -> Self {
        let plants_per_hectare: f64 = plants
            .iter()
            .map(|plant| plant.get_per_hectare() as f64)
            .sum();
        let dimension = map_dimension(polygon.get_area_hectare(), plants_per_hectare);
        let coords = Self::plantable_coords(dimension);

        let competition = get_competition_matrix(plants);
        let interaction = get_ecological_interaction_matrix(plants);
        let penalty = combine_matrices(&competition, &interaction);

        let radii: Vec<i32> = plants
            .iter()
            .map(|plant| plant.get_root_spread_radius())
            .collect();
        let max_radius = radii.iter().copied().max().unwrap_or(0);

        let index_of: HashMap<HexCoord, usize> = coords
            .iter()
            .enumerate()
            .map(|(index, &coord)| (coord, index))
            .collect();

        let mut adjacency = vec![Vec::new(); coords.len()];
        for (index, &coord) in coords.iter().enumerate() {
            for candidate in coord.within_radius(max_radius) {
                if candidate == coord {
                    continue;
                }
                if let Some(&neighbor) = index_of.get(&candidate) {
                    adjacency[index].push((neighbor, coord.distance(&candidate)));
                }
            }
        }

        TabuSearch {
            plants,
            area_hectare: polygon.get_area_hectare(),
            penalty,
            radii,
            coords,
            adjacency,
            config,
            row_sequence: Vec::new(),
            plants_per_hectare,
        }
    }

    pub fn with_row_sequence(mut self, sequence: Vec<usize>) -> Self {
        self.row_sequence = sequence;
        self
    }

    fn plantable_coords(dimension: usize) -> Vec<HexCoord> {
        let mut coords = Vec::with_capacity(dimension * dimension);
        for row in 0..dimension {
            for col in 0..dimension {
                coords.push(HexCoord::from_offset(col as i32, row as i32));
            }
        }
        coords
    }

    fn count_band(&self) -> CountBand {
        let ideals: Vec<f64> = self
            .plants
            .iter()
            .map(|plant| plant.get_per_hectare() as f64 * self.area_hectare)
            .collect();

        let target: Vec<usize> = ideals.iter().map(|&ideal| ideal.round() as usize).collect();
        let mut min: Vec<usize> = ideals
            .iter()
            .map(|&ideal| (ideal * (1.0 - COUNT_TOLERANCE)).floor() as usize)
            .collect();
        let mut max: Vec<usize> = ideals
            .iter()
            .map(|&ideal| (ideal * (1.0 + COUNT_TOLERANCE)).ceil() as usize)
            .collect();

        for index in 0..target.len() {
            min[index] = min[index].min(target[index]);
            max[index] = max[index].max(target[index]);
        }

        CountBand { target, min, max }
    }

    fn pair_cost(&self, a: i32, b: i32, distance: i32) -> f64 {
        if a < 0 || b < 0 {
            return 0.0;
        }
        let (a, b) = (a as usize, b as usize);
        let radius = self.radii[a].max(self.radii[b]);
        if distance <= radius {
            self.penalty[a][b]
        } else {
            0.0
        }
    }

    fn cost_with(&self, species: &[i32], index: usize) -> f64 {
        let current = species[index];
        if current < 0 {
            return 0.0;
        }

        self.adjacency[index]
            .iter()
            .map(|&(neighbor, distance)| self.pair_cost(current, species[neighbor], distance))
            .sum()
    }

    fn total_cost(&self, species: &[i32]) -> f64 {
        let directed: f64 = (0..species.len())
            .map(|index| self.cost_with(species, index))
            .sum();
        directed * 0.5
    }

    fn reassign_delta(&self, species: &[i32], index: usize, new_species: i32) -> f64 {
        let old_species = species[index];
        self.adjacency[index]
            .iter()
            .map(|&(neighbor, distance)| {
                self.pair_cost(new_species, species[neighbor], distance)
                    - self.pair_cost(old_species, species[neighbor], distance)
            })
            .sum()
    }

    pub fn solve(&self) -> Allocation {
        self.solve_with_budget(self.config.budget_per_hectare)
    }

    pub fn solve_with_budget(&self, budget_per_hectare: f64) -> Allocation {
        let dimension = map_dimension(self.area_hectare, self.plants_per_hectare);
        let capacity = self.coords.len();
        let plant_count = self.plants.len();
        let band = self.count_band();

        let mut rng = Rng::new(self.config.seed);
        let mut species = vec![-1i32; capacity];
        let mut counts = vec![0usize; plant_count];
        let mut pattern_species: Option<Vec<i32>> = None;
        let mut pattern_counts: Option<Vec<usize>> = None;

        if self.row_sequence.is_empty() {
            let mut cursor = 0;
            for (species_index, &target) in band.target.iter().enumerate() {
                for _ in 0..target {
                    if cursor >= capacity {
                        break;
                    }
                    species[cursor] = species_index as i32;
                    counts[species_index] += 1;
                    cursor += 1;
                }
            }
            let mut candidate = 0;
            while cursor < capacity {
                let mut advanced = false;
                for _ in 0..plant_count {
                    let current = candidate % plant_count;
                    candidate += 1;
                    if counts[current] < band.max[current] {
                        species[cursor] = current as i32;
                        counts[current] += 1;
                        cursor += 1;
                        advanced = true;
                        break;
                    }
                }
                if !advanced {
                    species[cursor] = 0;
                    counts[0] += 1;
                    cursor += 1;
                }
            }
            for index in (1..capacity).rev() {
                let other = rng.below(index + 1);
                species.swap(index, other);
            }
        } else {
            let sequence: Vec<usize> = self
                .row_sequence
                .iter()
                .copied()
                .filter(|&s| s < plant_count)
                .collect();
            let sequence = if sequence.is_empty() {
                (0..plant_count).collect()
            } else {
                sequence
            };
            let mut sequence_index = 0;
            for row in 0..dimension {
                let cols: Box<dyn Iterator<Item = usize>> = if row % 2 == 0 {
                    Box::new(0..dimension)
                } else {
                    Box::new((0..dimension).rev())
                };
                for col in cols {
                    let cell = row * dimension + col;
                    let species_index = sequence[sequence_index % sequence.len()];
                    sequence_index += 1;
                    species[cell] = species_index as i32;
                    counts[species_index] += 1;
                }
            }

            pattern_species = Some(species.clone());
            pattern_counts = Some(counts.clone());
            for deficient in 0..plant_count {
                while counts[deficient] < band.min[deficient] {
                    let donor = (0..plant_count)
                        .find(|&other| other != deficient && counts[other] > band.min[other]);
                    let Some(donor) = donor else { break };
                    let Some(cell) = species.iter().position(|&value| value == donor as i32) else {
                        break;
                    };
                    species[cell] = deficient as i32;
                    counts[donor] -= 1;
                    counts[deficient] += 1;
                }
            }

            for excess in 0..plant_count {
                while counts[excess] > band.max[excess] {
                    let receiver = (0..plant_count)
                        .find(|&other| other != excess && counts[other] < band.max[other]);
                    let Some(receiver) = receiver else { break };
                    let Some(cell) = species.iter().position(|&value| value == excess as i32)
                    else {
                        break;
                    };
                    species[cell] = receiver as i32;
                    counts[excess] -= 1;
                    counts[receiver] += 1;
                }
            }
        }

        let search_start_cost = self.total_cost(&species);
        let (display_species, initial_counts) = match pattern_species {
            Some(display) => (display, pattern_counts.unwrap_or_else(|| counts.clone())),
            None => (species.clone(), counts.clone()),
        };
        let initial_cost = self.total_cost(&display_species);
        let initial_species: Vec<Option<usize>> = display_species
            .iter()
            .map(|&value| {
                if value >= 0 {
                    Some(value as usize)
                } else {
                    None
                }
            })
            .collect();

        let mut best_cost = search_start_cost;
        let mut best = species.clone();
        let mut best_counts = counts.clone();

        let mut tabu: HashMap<(usize, i32), usize> = HashMap::new();
        let mut current_cost = search_start_cost;
        let mut iterations = 0;
        let mut improvements = 0;
        let mut tabu_hits = 0;
        let mut no_improve = 0;

        if capacity > 0 && plant_count > 0 {
            for iteration in 0..self.config.max_iterations {
                iterations = iteration + 1;

                let mut best_move: Option<(usize, i32)> = None;
                let mut best_delta = f64::INFINITY;

                for _ in 0..self.config.candidate_list_size {
                    let index = rng.below(capacity);
                    let current_species = species[index];
                    let new_species = rng.below(plant_count) as i32;
                    if new_species == current_species {
                        continue;
                    }

                    if current_species >= 0
                        && counts[current_species as usize] <= band.min[current_species as usize]
                    {
                        continue;
                    }
                    if new_species >= 0
                        && counts[new_species as usize] >= band.max[new_species as usize]
                    {
                        continue;
                    }

                    let is_tabu = tabu
                        .get(&(index, new_species))
                        .is_some_and(|&expiry| expiry > iteration);
                    let delta = self.reassign_delta(&species, index, new_species);
                    let aspirational = current_cost + delta < best_cost - 1e-12;

                    if is_tabu && !aspirational {
                        tabu_hits += 1;
                        continue;
                    }

                    if delta < best_delta - 1e-12 {
                        best_delta = delta;
                        best_move = Some((index, new_species));
                    }
                }

                match best_move {
                    Some((index, new_species)) => {
                        let old_species = species[index];
                        species[index] = new_species;
                        if old_species >= 0 {
                            counts[old_species as usize] -= 1;
                        }
                        if new_species >= 0 {
                            counts[new_species as usize] += 1;
                        }
                        current_cost += best_delta;
                        tabu.insert((index, old_species), iteration + self.config.tabu_tenure);

                        if current_cost < best_cost - 1e-12 {
                            best_cost = current_cost;
                            best = species.clone();
                            best_counts = counts.clone();
                            improvements += 1;
                            no_improve = 0;
                        } else {
                            no_improve += 1;
                        }
                    }
                    None => no_improve += 1,
                }

                if no_improve >= self.config.max_no_improve {
                    break;
                }
            }
        }

        let output_species = best
            .iter()
            .map(|&value| {
                if value >= 0 {
                    Some(value as usize)
                } else {
                    None
                }
            })
            .collect();

        let monetary_cost_total: f64 = (0..plant_count)
            .map(|i| self.plants[i].get_unitary_cost() * best_counts[i] as f64)
            .sum();
        let monetary_cost_per_hectare = if self.area_hectare > 0.0 {
            monetary_cost_total / self.area_hectare
        } else {
            0.0
        };

        Allocation {
            dimension,
            coords: self.coords.clone(),
            initial_species,
            initial_counts,
            species: output_species,
            counts: best_counts,
            target: band.target,
            min_counts: band.min,
            max_counts: band.max,
            initial_cost,
            best_cost,
            iterations,
            improvements,
            tabu_hits,
            monetary_cost_total,
            monetary_cost_per_hectare,
            budget_per_hectare,
            within_budget: monetary_cost_per_hectare <= budget_per_hectare + 1e-9,
        }
    }
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plants() -> Vec<Plant> {
        vec![
            Plant::new("A", 10.0, 40, [20, 30], [0, 20], 1, -0.5),
            Plant::new("B", 12.0, 60, [25, 35], [10, 30], 2, 0.5),
            Plant::new("C", 8.0, 30, [15, 25], [0, 15], 3, 0.0),
        ]
    }

    fn polygon() -> Polygon {
        Polygon::new("P1", 1.28, vec![0, 0, 0])
    }

    #[test]
    fn count_band_is_ideal_plus_minus_ten_percent() {
        let plants = plants();
        let polygon = polygon();
        let search = TabuSearch::new(&plants, &polygon, TabuConfig::default());
        let band = search.count_band();

        let ideals = [40.0_f64 * 1.28, 60.0 * 1.28, 30.0 * 1.28];
        for (index, ideal) in ideals.iter().enumerate() {
            let ideal = *ideal;
            assert_eq!(band.target[index], ideal.round() as usize);
            assert_eq!(
                band.min[index],
                (ideal * (1.0 - COUNT_TOLERANCE)).floor() as usize
            );
            assert_eq!(
                band.max[index],
                (ideal * (1.0 + COUNT_TOLERANCE)).ceil() as usize
            );
            assert!(band.min[index] <= band.target[index]);
            assert!(band.target[index] <= band.max[index]);
        }
    }

    #[test]
    fn solution_counts_stay_within_tolerance_band() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default()).solve();

        assert!(allocation.meets_requirements());
        for (index, &count) in allocation.counts.iter().enumerate() {
            assert!(count >= allocation.min_counts[index]);
            assert!(count <= allocation.max_counts[index]);
        }
    }

    #[test]
    fn initial_layout_is_recorded() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default()).solve();

        assert_eq!(allocation.initial_species.len(), allocation.coords.len());
        assert_eq!(
            allocation
                .initial_species
                .iter()
                .filter(|s| s.is_some())
                .count(),
            allocation.initial_counts.iter().sum::<usize>()
        );
        assert_eq!(
            allocation.initial_counts.iter().sum::<usize>(),
            allocation.coords.len()
        );
    }

    #[test]
    fn row_sequence_initialization_respects_band() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default())
            .with_row_sequence(vec![0, 1, 2])
            .solve();

        assert!(allocation.meets_requirements());
    }

    #[test]
    fn row_sequence_snakes_across_rows() {
        let plants = plants();
        let polygon = polygon();
        let sequence = vec![0usize, 1, 2];
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default())
            .with_row_sequence(sequence.clone())
            .solve();

        let dimension = allocation.dimension;
        let mut position = 0;
        for row in 0..dimension {
            let cols: Vec<usize> = if row % 2 == 0 {
                (0..dimension).collect()
            } else {
                (0..dimension).rev().collect()
            };
            for col in cols {
                let cell = row * dimension + col;
                assert_eq!(
                    allocation.initial_species[cell],
                    Some(sequence[position % sequence.len()])
                );
                position += 1;
            }
        }
    }

    #[test]
    fn solution_fills_every_cell_without_gaps() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default())
            .with_row_sequence(vec![0, 1, 2])
            .solve();

        assert!(allocation.species.iter().all(|species| species.is_some()));
        assert_eq!(allocation.placed(), allocation.coords.len());
    }

    #[test]
    fn search_never_worsens_best_cost() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default()).solve();

        assert!(allocation.best_cost <= allocation.initial_cost + 1e-9);
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let plants = plants();
        let polygon = polygon();
        let config = TabuConfig::default();
        let first = TabuSearch::new(&plants, &polygon, config).solve();
        let second = TabuSearch::new(&plants, &polygon, config).solve();

        assert_eq!(first.best_cost, second.best_cost);
        assert_eq!(first.species, second.species);
    }

    #[test]
    fn every_plant_maps_to_plants_index() {
        let plants = plants();
        let polygon = polygon();
        let allocation = TabuSearch::new(&plants, &polygon, TabuConfig::default()).solve();

        for species in allocation.species.iter().flatten() {
            assert!(*species < plants.len());
        }
    }
}
