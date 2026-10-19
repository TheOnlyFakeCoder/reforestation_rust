# Fase 4 — Convergencia del tabú, calibración y validación exacta: plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. TDD (prueba primero, verla fallar, implementar). Commits pequeños en español, **sin línea de atribución** (`Claude-Session` prohibido). Nunca `git push` ni merge.

**Goal:** Que el tabú termine por falta de mejora y no por tope en las 31 instancias, con parámetros calibrados y reportados; validar el tabú contra el óptimo exacto (CP-SAT) en instancias chicas; re-correr el pipeline y reportar qué cifras cambiaron.

**Architecture:** Rust (`src/`, CLI `reto` en `src/bin/reto.rs`) añade el motivo de paro y el tope escalado al motor tabú, un subcomando de calibración y un módulo `validacion.rs` que exporta instancias chicas, corre el tabú sobre ellas y evalúa asignaciones. Python (`python/`) añade `validacion_exacta.py` (CP-SAT) y extiende `figuras.py`/`resumen.py`.

**Tech Stack:** Rust estable (rand_chacha, serde_json, csv), Python `.venv` (numpy, pandas, matplotlib, ortools, pytest).

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` (§8 búsqueda tabú, §9 validación exacta) más los requisitos del usuario del 2026-10-18 (fase 4), resumidos en Global Constraints. Ecuaciones del reporte: (21) paso 1, (24) paso 3.

## Global Constraints

- Tope escalado: `max_iter_efectivo = max(tabu.max_iter, tabu.iter_por_posicion · |V|)`, `iter_por_posicion` en `config/default.json` → `tabu` (inicial 50). `max_sin_mejora_efectivo = max(tabu.max_sin_mejora, tabu.sin_mejora_por_posicion · |V|)`, `sin_mejora_por_posicion` inicial 0 (desactivado) y sólo se sube si las curvas lo justifican. |V| = `malla.len()` de la instancia.
- Criterio de aceptación de convergencia: sobre todas las búsquedas tabú del pipeline (paso 1 búsqueda y ajuste C, paso 3, práctica tabú) ≥ 90 % terminan `sin_mejora` en total y ≥ 80 % en cada instancia. Si no, subir `iter_por_posicion` en {50, 100, 200, 400} hasta cumplir.
- `motivo_paro` ∈ {`tope`, `sin_mejora`} (texto exacto en CSV). Una búsqueda con < 2 posiciones libres cuenta como `sin_mejora`.
- Calibración: instancias `1ha` y `map_21`, γ = γ* = 3, K (`candidatos`) ∈ {32, 64, 128} × tenure ∈ {10, 15, 25}, 5 semillas (`semilla + k`, k = 0..4). Salida `results/calibracion_tabu.csv`.
- Regla de elección de K y tenure: para cada instancia, z_rel = z_opt_media / min(z_opt_media de la rejilla); elegir la combinación con menor promedio de z_rel en las dos instancias; si otra queda a ≤ 0.001 de ese promedio y su tiempo medio es menor, elegir la más rápida.
- Validación exacta: malla tresbolillo con el `d` del config (4.189…), mismos anillos, h y W; rectángulos de 49 a 100 posiciones; coeficientes enteros = round(w·h·10 000) (θ también ×10 000); Z*, Z_TS y brecha con coeficientes originales; tolerancia de consistencia Rust↔Python 1e-6; CP-SAT 600 s por instancia, `num_workers = 8`; 10 semillas de tabú por instancia.
- No borrar archivos, funciones ni columnas existentes (se añaden columnas al final). `cargo test` normal y pytest en verde siempre.

## Review Focus

- Instancia de validación cuyo CP-SAT no cierra en 600 s → debe quedar en el CSV con `optimo_probado = False`, la cota y brecha contra la cota; nunca omitida (Task 6, prueba `test_no_optimo_se_reporta`).
- Z con pares existente-existente: Rust `z_total` los incluye; el modelo CP-SAT debe sumar esa constante para que Z* sea comparable (Task 6, prueba de consistencia con existentes).
- Instancias pequeñas con la vacía (paso 3 con huecos): la vacía no tiene peso; Python y Rust deben coincidir (Task 5 prueba `evaluar_con_vacias`, Task 6 fuerza bruta con huecos).
- Re-corrida: `--solo-gamma-star` y el barrido escriben las nuevas columnas sin romper a los lectores Python (`paso2_compra.py`, `figuras.py`, `resumen.py` leen por nombre de columna) (Task 1 y Task 7).
- `ui.rs` construye `ParamsTabu` con `config.tabu.params(...)`; debe seguir compilando y usar el tope escalado (Task 1).

---

### Task 1: Motivo de paro, tope escalado y desviación estándar (Rust)

**Files:** Modify `src/busqueda_tabu.rs`, `src/config.rs`, `src/paso1.rs`, `src/paso3.rs`, `src/reportes.rs`, `src/bin/reto.rs` (paso1, paso3, practica), `src/bin/ui.rs` (sólo la llamada de params), `config/default.json`. Tests: unitarias en esos módulos, `tests/fase2.rs`, `tests/fase3.rs`, `tests/cli.rs`.

**Interfaces — Produces:**
- `pub enum MotivoParo { Tope, SinMejora }` en `busqueda_tabu.rs` con `pub fn como_str(self) -> &'static str` ("tope"/"sin_mejora"); `Resultado.motivo_paro: MotivoParo`.
- `ConfigTabu.iter_por_posicion: usize` (default 50), `ConfigTabu.sin_mejora_por_posicion: usize` (default 0); `ConfigTabu::params_para(&self, semilla: u64, n_posiciones: usize) -> ParamsTabu` aplica la regla de escalado. `ConfigTabu::params` se queda igual (legacy de la UI) pero la UI pasa a `params_para`.
- `FilaPaso1.motivo_paro_paso1`, `FilaPaso1.motivo_paro_opt: MotivoParo`; `FilaPaso3.motivo_paro: MotivoParo`.
- CSV: `paso1_barrido_gamma.csv` y `paso1_gamma_star.csv` añaden al final `motivo_paro_paso1,motivo_paro_opt`; `paso1_resumen_gamma.csv` añade `z_opt_sd,paros_tope,paros_sin_mejora` (sd muestral n−1 sobre semillas; 0 con una semilla; conteos sobre las 2 búsquedas × semillas); `paso3_escenarios.csv` añade `motivo_paro`; `paso3_resumen.csv` añade `paros_tope,paros_sin_mejora` (ya trae `Z_sd` sobre escenarios); `practica_distribucion.csv` añade `motivo_paro` (vacío en las variantes sin tabú).

- [ ] **Step 1:** pruebas que fallan: (a) `buscar` con `max_iter = 50`, `max_sin_mejora = 10_000` → `MotivoParo::Tope`; con `max_sin_mejora = 5` y `max_iter = 100_000` → `SinMejora`; (b) `params_para(s, 1000)` con defaults da `max_iter = 50_000`, con `n = 100` da `20_000`; `sin_mejora_por_posicion = 2`, `n = 5000` → `max_sin_mejora = 10_000`; (c) `--set` con `tabu={...,"iter_por_posicion":100}` funciona y `iter_por_posicion = 0` es válido (desactiva el escalado); (d) CLI: `reto paso1` con `TABU_CORTO` escribe las columnas nuevas y valores en {tope, sin_mejora}; `reto paso3` escribe `motivo_paro` y `paros_*`.
- [ ] **Step 2:** verlas fallar (`cargo test`).
- [ ] **Step 3:** implementar; en `reto.rs` construir params por instancia con `params_para(semilla, instancia.malla.len())` en paso1, paso3 y practica.
- [ ] **Step 4:** `cargo test` y `cargo clippy --all-targets` en verde.
- [ ] **Step 5:** commits (motor; config; CSV/CLI).

### Task 2: Subcomando `reto calibrar` (Rust)

**Files:** Create `src/calibracion.rs` (+ `pub mod` en `lib.rs`); modify `src/bin/reto.rs`; tests en `tests/fase4.rs` (nuevo).

**Interfaces — Consumes:** `resolver_paso1(instancia, gamma, &ParamsTabu)`, `ConfigTabu::params_para` (Task 1). **Produces:** `pub struct FilaCalibracion { instancia, candidatos, tenure, n_semillas, z_opt_media, z_opt_sd, z_opt_mejor, objetivo_media, tiempo_medio_s, iter_media, frac_sin_mejora }`; `pub fn calibrar(instancia: &Instancia, gamma: f64, base: &ConfigTabu, candidatos: &[usize], tenures: &[usize], semillas: &[u64]) -> Vec<FilaCalibracion>`; `pub fn elegir(filas: &[FilaCalibracion]) -> (usize, usize)` con la regla de Global Constraints. CLI `reto calibrar [--instancias 1ha,map_21]` escribe `results/calibracion_tabu.csv` (una fila por instancia × K × tenure; tiempos = paso 1 + ajuste C; iteraciones = suma de ambas; `frac_sin_mejora` sobre las 2 búsquedas × semillas) e imprime la elección.

- [ ] **Step 1:** pruebas: `elegir` sobre filas sintéticas (gana menor z_rel promedio; empate ≤ 0.001 → más rápida); `calibrar` con rejilla 2×1, 2 semillas y `TABU_CORTO` sobre 1ha devuelve 2 filas con `z_opt_sd ≥ 0` y `z_opt_mejor ≤ z_opt_media`; CLI escribe el CSV con el encabezado exacto.
- [ ] **Step 2–4:** fallar, implementar (paralelizar por combinación con `std::thread::scope` como en paso3), pasar.
- [ ] **Step 5:** commit.

### Task 3: Correr calibración y criterio de convergencia; fijar parámetros (ejecución)

**Files:** `config/default.json`, `results/calibracion_tabu.csv`, `results/convergencia_criterio.csv` (nuevo, escrito por `python/convergencia.py`, nuevo, con prueba).

- [ ] **Step 1:** `python/convergencia.py` lee `paso1_gamma_star.csv`, `paso1_barrido_gamma.csv`, `paso3_escenarios.csv`, `practica_distribucion.csv` y escribe por instancia `busquedas, paros_tope, paros_sin_mejora, frac_sin_mejora` más una fila `total`; sale con código 1 si no cumple el criterio. Prueba pytest sobre CSV sintéticos (cumple / no cumple).
- [ ] **Step 2:** con `iter_por_posicion = 50`: `reto paso1 --instancias todas --solo-gamma-star`, `reto montecarlo`, `paso2_compra.py`, `reto paso3 --instancias todas`, `reto practica --instancias todas`, `convergencia.py`. Si no cumple, subir a 100, 200, 400 y repetir. Si con 400 no cumple, parar y reportar (DONE_WITH_CONCERNS) con las curvas de map_21.
- [ ] **Step 3:** `reto calibrar`; aplicar la elección (K, tenure) a `config/default.json`; re-verificar el criterio con los nuevos K, tenure (Step 2 otra vez).
- [ ] **Step 4:** commit de config + `calibracion_tabu.csv` + script; el informe lista el `iter_por_posicion` final, K, tenure y el tiempo medio por corrida en 1ha, map_27 y map_21.

### Task 4: Figuras y RESUMEN de convergencia (Python)

**Files:** `python/figuras.py`, `python/resumen.py`, `python/tests/`.

- `convergencia` también para `map_21` (añadir la instancia sólo a esa figura).
- RESUMEN: tabla por instancia de motivos de paro (de `convergencia_criterio.csv`), `z_opt_sd` en la tabla del paso 1, tabla de calibración y los parámetros finales del tabú (de `config/default.json`), nota de que `Z_sd` del paso 3 es sobre escenarios (una semilla por escenario).
- [ ] Pruebas primero (secciones y un número cada una), implementar, regenerar, commit.

### Task 5: Instancias de validación en Rust (`validacion.rs`)

**Files:** Create `src/validacion.rs`; modify `src/lib.rs`, `src/bin/reto.rs`; tests en `tests/fase4.rs`.

**Interfaces — Produces:** formato JSON `results/validacion/inst_<id>.json`:
`{id, modelo: "paso1"|"paso3", columnas, filas, disposicion, n_pos, posiciones: [[x,y]], aristas: [[u,v,anillo]] (u<v), peso: [anillo][j][k] (= tabla.peso(anillo, j+1, k+1)), fija: [-1|j] por posición (−1 = libre), cuotas: [n_j] | null, m: M | null, banda_l, banda_u, d_tot | null, theta: [θ_j], gamma, semilla_existentes}`.
Especies 0..9 en JSON/CSV; −1 = vacía.
- `reto instancias-val` escribe estas 10 instancias (semilla del config):

| id | modelo | columnas × filas (Completa) | existentes | caso |
|---|---|---|---|---|
| p3_a | paso3 | 7×7 = 49 | 0 | sin existentes, M = |V| |
| p3_b | paso3 | 9×9 = 81 | 0 | sin existentes, M = |V| |
| p3_c | paso3 | 7×7 = 49 | 20 % | normal: n = C⁺, M = |V_lib| |
| p3_d | paso3 | 8×8 = 64 | 20 % | normal |
| p3_e | paso3 | 10×10 = 100 | 20 % | normal |
| p3_f | paso3 | 8×8 = 64 | 20 % | huecos: n_j = round(0.9·C⁺_j), M = Σn_j < |V_lib| |
| p3_g | paso3 | 10×10 = 100 | 20 % | huecos |
| p1_a | paso1 | 7×7 = 49 | 0 | D_tot = |V|, banda ε del config |
| p1_b | paso1 | 8×8 = 64 | 0 | idem |
| p1_c | paso1 | 10×10 = 100 | 0 | idem |

D = composición nominal (mayor residuo) de |V|; existentes: round(0.2·|V|) posiciones al azar sin reemplazo con especies ~ Mult(T, p̂) (`mc_params.json`); C = D − X truncado a 0; en paso 3 sin existentes n = D. Para el tabú del paso 3 la solución inicial asigna las especies en serpentina sobre las libres hasta cumplir n y M.
- `reto tabu-val [--semillas 10]`: para cada JSON corre el tabú (paso 1: `resolver_paso1`-equivalente con γ = 3 sobre esta malla; paso 3: `Restriccion::Cuotas`) con `params_para` de los parámetros finales y semillas `semilla + k`; escribe `results/validacion/tabu_<id>.csv` (`semilla, z, objetivo, tiempo_s, iteraciones, motivo_paro, phi` con phi como lista separada por `;`).
- `reto evaluar-val --inst <json> --phi <csv>`: imprime `z` y `objetivo` (γZ + Σθ·D en paso 1; Z en paso 3) de la asignación con 12 decimales.
- [ ] **Step 1:** pruebas: JSON de p3_c tiene |V| = 49, ~10 fijas, Σn ≥ M = |V_lib|; p3_f tiene M < |V_lib|; aristas = pares por anillo de la malla; `evaluar` de una φ conocida = `z_total` (incluye pares existente-existente; vacías sin peso); `tabu-val` respeta cuotas, fijas y M.
- [ ] **Step 2–4:** fallar, implementar, pasar. **Step 5:** commits.

### Task 6: `python/validacion_exacta.py` (CP-SAT)

**Files:** Create `python/validacion_exacta.py`, `python/tests/test_validacion_exacta.py`; modify `run_all.ps1`, `README.md`, `python/resumen.py`.

**Interfaces — Consumes:** JSON, `tabu_<id>.csv` y `reto evaluar-val` de Task 5.
- `z_python(inst, phi) -> float` y `objetivo_python(inst, phi) -> float` (coeficientes originales, incluye pares con existentes y existente-existente).
- `modelo_cpsat(inst, escala=10_000) -> (CpModel, x)`: `x[v][j]` booleanas para v libre; paso 3: Σ_j x_vj ≤ 1, Σ_v x_vj ≤ n_j, Σ x = M; paso 1: Σ_j x_vj = 1, L_j ≤ Σ_v x_vj ≤ U_j. Arista libre-libre: `y ≥ x_uj + x_vk − 1` (y booleana) por par (j,k) con coeficiente > 0; libre-fija: término lineal sobre x; fija-fija: constante. Objetivo entero con coeficientes round(·×escala).
- `resolver(inst, tiempo=600, workers=8) -> dict` con Z*, cota, óptimo probado, tiempo, n_variables, error máximo de redondeo (max |round(c·e)/e − c|).
- Consistencia antes de comparar: Z de Python de cada solución tabú = Z de Rust (1e-6); Z de CP-SAT evaluada con `reto evaluar-val` = Z de Python (1e-6). Si falla: escribir la discrepancia y salir con código 2 (el controlador se detiene y avisa al usuario).
- `results/validacion_exacta.csv`: `id, modelo, n_pos, existentes, aristas, variables, z_opt, cota, optimo_probado, tiempo_cpsat_s, z_ts_mejor, z_ts_media, z_ts_sd, tiempo_ts_medio_s, brecha_mejor, brecha_media, frac_semillas_optimo, error_redondeo_max` (para paso 1 `z_*` es el objetivo γZ + Σθ·D; brecha = (Z_TS − Z*)/Z*; sin óptimo probado, brecha contra la cota y se marca). `frac_semillas_optimo`: semillas con |Z_TS − Z*| ≤ 1e-6·max(1, Z*).
- RESUMEN: sección "Validación exacta" con la tabla y un párrafo generado sólo de los números (cuántas óptimas probadas, brecha máxima y media, fracción de semillas que alcanzan el óptimo).
- `run_all.ps1`: tras `practica`, `reto instancias-val`, `reto tabu-val`, `validacion_exacta.py`, con un switch `-OmitirValidacion`. README: los comandos.
- [ ] **Step 1:** pruebas (instancias sintéticas de 9 posiciones, 3×3, con 3 especies activas y aristas/pesos inventados): CP-SAT = fuerza bruta en paso 1 y paso 3 (con y sin huecos, con una fija); el argmin con escala ×10 000 evaluado con coeficientes originales iguala el óptimo original; una instancia con `tiempo` diminuto devuelve `optimo_probado=False` y queda en el CSV (`test_no_optimo_se_reporta`).
- [ ] **Step 2–4:** fallar, implementar, pasar. **Step 5:** commits. **Step 6:** correr la validación completa (en segundo plano; hasta ~100 min) y commitear `validacion_exacta.csv`, los JSON/CSV de `results/validacion/` y RESUMEN.

### Task 7: Re-corrida completa y tabla de cambios

- [ ] Guardar copia de los CSV actuales (`git show HEAD:results/...`) antes de re-correr.
- [ ] `run_all.ps1` completo (con validación) con los parámetros finales.
- [ ] `python/cambios.py` (con prueba mínima) → `results/cambios_fase4.md`: antes/después de Z_opt del paso 1 (1ha, map_27, map_21), D* (si cambió), Z media del paso 3 (1ha, map_27, map_21), compra q y costo del paso 2 (1ha, map_27), reducción de Z frente a la serpentina iii (1ha, map_27), más la tabla de validación y los parámetros finales del tabú. La versión "antes" se lee de `git show <commit base>:results/...`.
- [ ] Commit de resultados y tabla.
