# Reforestation

A Rust workspace for planning species placement on reforestation polygons. Given a set of
plant species and their per-hectare targets, it produces a gapless planting map that minimizes
ecological competition between neighbouring plants while respecting per-species count limits and
a monetary budget.

This is a Rust port of the C++ reference in `../reforestation`.

## What it does

- **Gapless dense grid.** The map is sized so its cell count matches the required density of
  `658 plants/ha` (the sum of the species `per_hectare` values). Every grid cell is plantable and
  filled, so there are no gaps.
- **Snake initial pattern.** Starting from a user-supplied species sequence, the layout is seeded
  in boustrophedon (snake) order: each row is the direct continuation of the previous one,
  alternating direction (right then left then right...).
- **Tabu Search.** Reassignment moves relocate plants between species to lower the competition
  cost, guided by tabu tenure and aspiration.
- **Count band.** Each species stays within +/-10% of its ideal count (`per_hectare * area`),
  with the total filling the whole map.
- **Budget tracking.** Monetary cost per hectare (`sum(count * unitary_cost) / area`) is compared
  against a `12,500 $/ha` budget.
- **Interactive UI.** An `egui`/`eframe` dashboard with a Before/After view toggle, live stats,
  per-species counts, and a hex map coloured by species.

## Layout

| Path | Purpose |
| --- | --- |
| `src/reforestation.rs` | `Plant`, `Polygon`, `Wasted` data types and binary serialization |
| `src/map.rs` | Grid sizing (`map_dimension`) and map construction |
| `src/hex_grid.rs` | Axial hex coordinates (`HexCoord`) and offset conversions |
| `src/interaction.rs` | Ecological interaction matrix, penalties, competitive cost matrix |
| `src/competition.rs` | Competition coefficient matrix |
| `src/terrain.rs` | `TerrainMap` (occupied/plantable slots) |
| `src/tabu.rs` | `TabuConfig`, `Allocation`, `TabuSearch` optimizer |
| `src/algorithm.rs`, `src/matrix_handler.rs` | Reference numerical helpers |
| `src/writer.rs` | Serialization helpers |
| `src/bin/simulation.rs` | Runs the optimizer over all polygons and prints a detailed report |
| `src/bin/ui.rs` | Interactive dashboard |
| `src/bin/writer.rs` | Generates the sample data files in `bins/` |
| `bins/` | Sample `plants.bin`, `polygons.bin`, `steadyValues.bin` |

## Requirements

- Rust (edition 2021)
- Linux with X11 or Wayland for the UI (uses `glow`)

## Build and test

```sh
cargo build
cargo test
cargo clippy --all-targets
cargo fmt
```

## Run

Generate the sample data files (only needed once, or after changing the inputs):

```sh
cargo run --bin writer
```

Run the optimizer over every polygon and print a per-polygon summary plus a detailed report for
the first polygon (per-species counts, initial snake pattern, optimized grid):

```sh
cargo run --bin simulation
```

Launch the interactive UI:

```sh
cargo run --bin ui
```

## UI overview

- **Left panel:** run the selected polygon or all polygons (background thread), pick a polygon,
  filter by species, and inspect an individual cell.
- **Center:** summary statistics, the species count table, and the grid. Use the **Before / After**
  toggle to switch between the initial snake pattern and the optimized layout; the statistics
  (placed, eco cost, `$`/ha and budget status) follow the selected view.

## Algorithm notes

- Density: grid dimension is `round(sqrt(plants_per_hectare * area))`.
- Ideal per species: `per_hectare * area`; band is `[floor(0.9 * ideal), ceil(1.1 * ideal)]`.
- Moves reassign a cell from one species to another (never to empty), keeping counts inside the
  band. The objective is the pairwise competition cost within each plant's root-spread radius.
- Defaults: `max_iterations = 10000`, `max_no_improve = 60`, `tabu_tenure = 12`,
  `candidate_list_size = 256`, fixed seed for deterministic runs.

The budget (`12,500 $/ha`) is a reporting/constraint signal: at the required counts every polygon
is currently over budget (roughly `15,200-15,600 $/ha`).

## License

MIT. See [LICENSE](LICENSE).
