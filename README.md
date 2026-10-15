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

## Platform support

The project builds and runs on Linux, macOS, and Windows.

- The library and the `simulation` / `writer` binaries are pure Rust and OS-independent.
- The `ui` binary uses `eframe`/`egui` with the `glow` (OpenGL) backend, which works on all three
  platforms.
- Data files are read from `bins/*.bin` using relative paths, so run the binaries from the project
  root.

Note: `Cargo.toml` currently enables the `x11` and `wayland` features on `eframe`. Those are
Linux-only winit backends; they are forwarded to winit/glutin which target-gate them, so they are
no-ops on Windows/macOS and the build still works. To be explicit, they can be gated to Linux:

```toml
[dependencies]
eframe = { version = "0.36", default-features = false, features = ["glow", "default_fonts"] }

[target.'cfg(target_os = "linux")'.dependencies]
eframe = { version = "0.36", features = ["x11", "wayland"] }
```

## Setup per OS

### Linux

1. Install Rust (edition 2021 toolchain):

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install the system libraries needed by the UI (X11/Wayland + OpenGL). For Debian/Ubuntu:

   ```sh
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config \
     libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxinerama-dev \
     libgl1-mesa-dev libegl1-mesa-dev libwayland-dev libxkbcommon-dev
   ```

   Fedora:

   ```sh
   sudo dnf install -y gcc-c++ pkgconf-pkg-config \
     libX11-devel libXcursor-devel libXrandr-devel libXi-devel libXinerama-devel \
     mesa-libGL-devel mesa-libEGL-devel wayland-devel libxkbcommon-devel
   ```

3. Build and run (see below). The UI needs an X11 or Wayland session (`$DISPLAY` or
   `$WAYLAND_DISPLAY` set).

### macOS

1. Install the Xcode command line tools (provides the linker and system SDK):

   ```sh
   xcode-select --install
   ```

2. Install Rust:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build and run (see below). No extra system libraries are required. The `glow` backend uses
   OpenGL, which Apple has deprecated but still supports; to use Metal instead, switch the UI to
   the `wgpu` backend later.

### Windows

1. Install the Microsoft C++ build tools: download **Visual Studio Build Tools** and select the
   **Desktop development with C++** workload (provides the MSVC linker).
2. Install Rust with the default MSVC toolchain: download and run `rustup-init.exe` from
   <https://rustup.rs> (host triple `x86_64-pc-windows-msvc`).
3. Build and run (see below) from PowerShell or `cmd.exe`. No extra system libraries are required.

## Build and test

Identical on all platforms:

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

The `cargo run ...` commands are the same in PowerShell/`cmd.exe` on Windows.

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
