# Reforestación CONAFOR: Entregable 2 (Optimización Estocástica)

Modelo de reforestación con competencia entre especies: composición de especies (paso 1),
compra de plantas bajo incertidumbre (paso 2), ajuste al polígono real con plantas existentes
(paso 3) y comparación con la práctica actual. Los pasos 1 y 3 y la práctica están en Rust
(búsqueda tabú); el ajuste de distribuciones, la compra (PLE) y las figuras están en Python.
Los resultados están en `results/` (resumen en `results/RESUMEN.md`) y las figuras en `figures/`.

## Requisitos

- Rust estable (`cargo`).
- Python con un entorno `.venv` en la raíz del repositorio:

```powershell
python -m venv .venv
.venv\Scripts\python.exe -m pip install numpy scipy pandas matplotlib pulp highspy ortools pytest
```

(también están en `requirements.txt`).

## Estructura

| Ruta | Contenido |
| --- | --- |
| `src/` | Módulos del modelo en Rust: `config`, `datos`, `malla`, `competencia`, `objetivo`, `composicion`, `busqueda_tabu`, `paso1`, `escenarios`, `paso3`, `practica`, `reportes`; `src/bin/reto.rs` es el CLI |
| `python/` | `comun.py`, `ajuste.py`, `paso2_compra.py`, `practica_compra.py`, `markov.py`, `figuras.py`, `resumen.py`, `validacion_exacta.py`, `transicion.py`, `entorno.py`, `tamano.py`, `experimentacion.py`, `sensibilidad.py`, `cambios.py` y `tests/` |
| `run_all.ps1`, `pasos.ps1` | Pipeline completo; `pasos.ps1` define `Invoke-Paso` (corre y mide cada paso) |
| `config/default.json` | Parámetros compartidos por Rust y Python (γ* = 3, ε = 0.1, π_factor = 2, tabú, semillas) |
| `data/` | Datos del reto (los escribe `reto datos`) |
| `results/` | CSV/JSON de cada paso y `RESUMEN.md` |
| `figures/` | Figuras PNG (300 dpi) |

## Pipeline completo

Todo en orden con `.\run_all.ps1`. Paso a paso, desde la raíz en PowerShell:

```powershell
cargo build --release --bin reto
$reto = ".\target\release\reto.exe"
$py = ".venv\Scripts\python.exe"

& $reto datos
& $reto malla
& $py python\ajuste.py
& $reto paso1 --instancias 1ha,map_27
& $reto paso1 --instancias todas --solo-gamma-star
& $reto montecarlo --instancias todas
& $py python\paso2_compra.py
& $reto paso3 --instancias todas      # 1000 escenarios en 1ha y map_27, 100 en las demás
& $reto practica --instancias todas   # incluye DSATUR sin y con existentes
& $py python\transicion.py          # orden_plantacion_<inst>.csv, transicion.csv, transicion_estacionaria.csv
& $py python\entorno.py             # entorno.json / entorno.md (PC, RAM, disco, versiones, commit)
& $py python\tamano.py              # tamano_modelo.csv, tamano_parametros.csv, tamano_problema.csv
& $py python\experimentacion.py     # experimentacion.csv (valores esperados por ha con IC 95 %)
& $py python\sensibilidad.py        # results/sensibilidad/<variante>/ y sensibilidad.csv
& $reto instancias-val              # results/validacion/inst_<id>.json
& $reto tabu-val                    # results/validacion/tabu_<id>.csv (10 semillas)
& $py python\validacion_exacta.py   # results/validacion_exacta.csv (CP-SAT)
& $py python\practica_compra.py
& $py python\markov.py
& $py python\figuras.py
& $py python\resumen.py
```

Después de actualizar el código (`git pull`) hay que recompilar `reto.exe`
(`cargo build --release --bin reto`) antes de correr pasos sueltos; `run_all.ps1` lo hace como
primer paso, salvo con `-SoloValidacion`.

Opciones de `run_all.ps1`:

- `-OmitirValidacion`: salta los tres pasos de la validación exacta.
- `-OmitirSensibilidad`: salta `python\sensibilidad.py` (9 variantes de la cadena completa en
  1ha y map_27, más el paso 1 en map_21).
- `-SoloValidacion`: corre sólo `reto instancias-val`, `reto tabu-val` y
  `python\validacion_exacta.py` (no compila).
- `-ArgumentosValidacion @(...)`: argumentos para `python\validacion_exacta.py` (por defecto
  ninguno), p. ej.
  `.\run_all.ps1 -SoloValidacion -ArgumentosValidacion @('--formulaciones', 'ajustada', '--workers', '32', '--tiempo', '600', '--actualizar')`.

Cada paso se mide con `Measure-Command` y se añade una fila a `results/tiempos.csv`
(`paso, instancias, segundos, fecha`; `instancias` es el valor de `--instancias` o vacío). El
archivo acumula las corridas y `RESUMEN.md` muestra la última de cada paso en
"Entorno y software". `Invoke-Paso` vive en `pasos.ps1` (parámetro `-Tiempos` para otra ruta).

Salidas nuevas de la entrega final: `entorno.json`/`entorno.md`, `tamano_*.csv`,
`experimentacion.csv`, `orden_plantacion_<inst>.csv`, `paso3_transicion_<inst>.csv`,
`transicion.csv`, `transicion_estacionaria.csv`, `sensibilidad.csv`, `tiempos.csv` y las figuras
`orden_plantacion_<inst>.png` y `sensibilidad.png`. `RESUMEN.md` las reúne en las secciones
"Entorno y software", "Tamaño del problema", "Experimentación (1000 simulaciones)", "Orden de
plantación y transiciones" y "Sensibilidades".

CP-SAT corre
600 s por instancia con todos los hilos lógicos (`--workers`, por defecto `os.cpu_count()`) en la
formulación básica y, si no prueba el óptimo, otros 600 s con la ajustada (10 instancias: hasta unas
3.5 h). `python\validacion_exacta.py` acepta `--tiempo S`, `--workers N`, `--ids p3_a,p1_c`,
`--formulaciones ajustada` (en orden; la siguiente sólo si la anterior no cierra), `--actualizar`
(conserva las filas de `--salida` que no se vuelven a correr), `--dir`, `--salida` y `--reto`. El
CSV actual tiene la básica con 8 workers y la ajustada vuelta a correr con 32:
`& $py python\validacion_exacta.py --formulaciones ajustada --workers 32 --actualizar`. Antes de comparar
verifica que Python y `reto evaluar-val` den el mismo Z (1e-6); si no, escribe
`results/validacion_exacta_discrepancias.txt` y sale con código 2. Para evaluar asignaciones a
mano: `& $reto evaluar-val --inst results\validacion\inst_p3_a.json --phi CSV` (columna `phi` con
especies separadas por `;` y −1 para vacía, o una columna `especie` por posición).

## Cambiar parámetros

Los subcomandos de `reto` aceptan:

- `--datos DIR` y `--resultados DIR`: carpetas de entrada y salida (por defecto `data` y `results`).
- `--config RUTA`: archivo JSON de configuración (por defecto `config/default.json`).
- `--set clave=valor`: sobrescribe un parámetro; se puede repetir. Ejemplo:
  `& $reto paso1 --instancias 1ha --set gamma_star=1 --set n_semillas=1`.
  Al bajar `n_escenarios` hay que bajar también `n_escenarios_paso3` y vaciar
  `escenarios_paso3_por_instancia` (la validación exige que los escenarios del paso 3 no pasen de
  `n_escenarios`):
  `--set n_escenarios=10 --set n_escenarios_paso3=5 --set "escenarios_paso3_por_instancia={}"`.
- `--instancias LISTA`: `1ha`, `map_<id>` separados por comas, o `todas`
  (por defecto `1ha,map_27`).
- `--solo-gamma-star` (sólo `paso1`): corre únicamente γ* en vez del barrido de γ.
- `--compra CSV` (sólo `paso3`): compra del paso 2 (por defecto `<resultados>/paso2_compra.csv`).

`python\figuras.py` acepta nombres de figuras (`mapa`, `practica_distribucion`, ...) y
`--instancias`; `python\resumen.py` acepta `--origen` y `--salida`.

## Pruebas

```powershell
cargo test
.venv\Scripts\python.exe -m pytest python/tests
```

## Otros binarios

`ui` es una interfaz gráfica (egui) y `validar_paso2` compara el paso 2 en Rust con los resultados;
el paso 2 oficial es `python\paso2_compra.py` (PuLP + HiGHS).

Las cifras de `ui` (como máximo 10 escenarios, con respaldos cuando faltan archivos) son ilustrativas y no
sustituyen a `results/`. `reto paso3` sólo corre con pares (ε, π) presentes en `paso2_compra.csv`: ε entre los
valores escritos y `pi_factor` en {2, 4, 10}.

---

# Legacy (simulación original)

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

At startup the UI loads the selected polygon from the published pattern
(`results/paso1_patron_<name>.csv`), so it is ready in milliseconds — the status bar shows
`paso 1 CSV`. **Run selected** / **Run all** do the same instant refresh by default; tick
**Force tabú re-solve** (next to the buttons) to re-run the optimizer instead — status then shows
`tabú`, and that solve is the slow part, so prefer `--release` for it:

```sh
cargo run --release --bin ui
```

The `cargo run ...` commands are the same in PowerShell/`cmd.exe` on Windows.

## UI overview

- **Left panel:** run the selected polygon or all polygons (background thread), pick a polygon,
  filter by species, and inspect an individual cell.
- **Center:** summary statistics, the species count table, and the grid. Use the **Before / After**
  toggle to switch between the initial snake pattern and the optimized layout; the statistics
  (placed, eco cost, `$`/ha and budget status) follow the selected view. The status bar reports
  where each result came from (`paso 1 CSV` = published pattern, `tabú` = solved in the UI), and
  the stats line shows the iterations/time of that run.

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
