# Fase 5 — Entrega final (experimentación y resultados): plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (o executing-plans). TDD (prueba primero, verla fallar, implementar). Commits pequeños en español, **sin línea de atribución** (`Claude-Session` prohibido). Nunca `git push` ni merge.

**Goal:** Producir todo lo que pide la entrega final (entorno, tiempos, tamaño del problema, 1000 simulaciones con IC, competencia esperada, matrices de transición sobre todos los escenarios, orden de plantación) más lo que se había recortado (DSATUR, sensibilidades), y cerrar la fase 4 (tabla de cambios, validación con 32 workers), con una sola tanda de corridas largas al final.

**Architecture:** Rust: `paso3` lee el número de escenarios por instancia del config y, con una función nueva junto a `correr_instancia`, acumula conteos de transición (vecindad inmediata y orden de siembra) y Z por nivel por escenario; módulos nuevos `transicion.rs` y `dsatur.rs` (variantes nuevas en `reto practica`). Python: scripts pequeños que leen `results/` (`cambios.py`, `transicion.py`, `entorno.py`, `tamano.py`, `experimentacion.py`, `sensibilidad.py`) y extensiones de `figuras.py`/`resumen.py`. `run_all.ps1` mide cada paso en `results/tiempos.csv`.

**Tech Stack:** Rust estable (csv, serde_json), Python `.venv` (numpy, pandas, matplotlib, pulp/highspy, ortools, pytest), PowerShell 5.1.

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` (§2 W, §3 alcance, §5 paso 1, §7 paso 3, §10 práctica y DSATUR, §11 Markov) y la adenda `docs/superpowers/specs/2026-10-20-fase5-entrega-final.md` (requisitos de la entrega, decisiones del usuario y ajustes del 2026-10-20 al plan). Ledger de la fase 4: `.superpowers/sdd/2026-10-18-fase4-convergencia-validacion/progress.md`.

## Global Constraints

- No borrar archivos, funciones ni columnas existentes; columnas nuevas al final; variantes nuevas como filas nuevas. Lo reemplazado queda marcado LEGACY o al lado (p. ej. `correr_instancia` se conserva con la misma firma y comportamiento).
- `cargo test` normal (sin reporter ni `RUSTC_BOOTSTRAP`) y `.venv\Scripts\python.exe -m pytest -q python/tests` en verde en cada commit.
- Las pruebas no escriben en `results/` (usan `tmp_path` / directorios temporales).
- Ninguna tarea de implementación (1–8) corre pasos largos sobre `results/`; las corridas sobre `results/` sólo en la Task 9. Las corridas cortas de prueba van a un directorio temporal o al scratchpad.
- Codificación: φ interno 0 = vacía, especie j ↔ j+1; CSV con `especie` 0..9 y −1 vacía.
- Instancias: `1ha` y `map_27` (= P1); `map_21` = polígono de 8 ha para la comprobación de sensibilidad.
- Paso 3: 1000 escenarios en `1ha` y `map_27`, 100 en las demás.
- Orden de siembra en serpentina: filas por `fila` creciente; dentro de la fila, `col` creciente si `fila` es par y decreciente si es impar; sólo plantas nuevas (φ > 0 y no existente). Vecindad inmediata: pares de posiciones a distancia d (anillo 0 de `Grafo`, ≡ distancia ≤ d + tol), ambas plantadas (nuevas o existentes), contadas en los dos sentidos. Rust y Python usan exactamente estas reglas.
- Formato de matrices de transición (largo): `de, a, conteo, n, prob` con j, k ∈ 0..9 (100 filas por matriz); `n` = Σ conteos de la fila; `prob = conteo/n`, o 0 si `n = 0`.
- IC 95 % de una media: media ± 1.96·sd/√n (sd muestral, n escenarios).
- Sensibilidades: variantes exactas `base`, `kappa_0.4` (`kappa=0.4`), `kappa_0.8`, `radios_0.75` (`factor_radios=0.75`), `radios_1.25`, `m_0` (`m_escala=0`), `m_0.5`, `m_2`, `s2` (`usar_s2=true`); cadena completa en 1ha y map_27 con **100** escenarios del paso 3 en todas las variantes (incluida `base`); sólo paso 1 γ* en map_21.
- Tabla de cambios de la fase 4: "antes" = `be65ce0`, "después" = `45c370f`.

## Review Focus

- Que ω = 0 (patrón y fila) no dependa del número de escenarios: de él dependen práctica, DSATUR y la tabla de distribución (Task 2, `omega0_no_depende_de_n`; Task 9 lo comprueba sobre datos reales antes de re-correr y se detiene si difiere).
- Reglas de orden y vecindad idénticas en Rust y Python: mismas cifras sobre el mismo patrón (Tasks 3 y 5 comparten el caso fijo de 2×3; Task 9 compara Rust con 1 escenario contra Python sobre ω = 0 real).
- DSATUR con Σ cuotas < posiciones libres (presupuesto corto): quedan vacías exactamente las sobrantes y las cuotas se cumplen (Task 4, `dsatur_respeta_cuotas_con_huecos`).
- `m_escala = 0` (θ = 0) y valores del mapa de escenarios > `n_escenarios` (Monte Carlo genera menos): la primera se acepta y corre; la segunda es error claro de validación (Task 2 `validar_rechaza_paso3_mayor_que_mc`, Task 7 `test_config_variante_m0_valida`).
- Especie ausente en una matriz (fila con n = 0): prob 0, sin NaN, y la estacionaria se calcula sólo sobre las especies presentes (Task 5, `test_fila_sin_ocurrencias`, `test_estacionaria_con_especie_ausente`).

---

### Task 1: Tabla de cambios de la fase 4 (`python/cambios.py`)

**Files:** Create `python/cambios.py`, `python/tests/test_cambios.py`.

**Interfaces — Produces:** `leer_git(commit: str, ruta: str) -> pd.DataFrame` (CSV de `git show <commit>:<ruta>`); `tabla_cambios(antes: dict[str, pd.DataFrame], despues: dict[str, pd.DataFrame]) -> pd.DataFrame` con columnas `cifra, instancia, antes, despues, cambio_pct`; claves de los dicts: `paso1_gamma_star`, `paso1_composicion`, `paso3_resumen`, `paso2_compra`, `practica_distribucion`; `main(antes="be65ce0", despues="45c370f", salida="results/cambios_fase4.md")`, CLI `--antes --despues --salida`.

- [ ] **Step 1: Pruebas** con DataFrames sintéticos: `test_tabla_cambios_z_opt` (media de z_opt sobre semillas en γ* = 3: 100 → 99 en `1ha` → fila `"Z_opt paso 1"`, `cambio_pct == -1.0`); `test_tabla_cambios_reduccion_serpentina` (Z iii 200, tabú con existentes 150 → `"reducción vs serpentina iii"`, `despues == 25.0` en %); `test_tabla_cambios_compra` (sólo ε = 0.1, π_factor = 2); `test_d_star_sin_cambio` (D* igual → `antes == despues`, texto `"[…]"`).
- [ ] **Step 2:** ver fallar. **Step 3:** implementar. Cifras: Z_opt paso 1 y D* (1ha, map_27, map_21); Z media paso 3 (1ha, map_27, map_21); costo y q del paso 2 (1ha, map_27); reducción % de `tabu_con_existentes` frente a `iii_serpentina_existentes` (1ha, map_27). `main` escribe Markdown: la tabla, la tabla de validación (filas definitivas de `results/validacion_exacta.csv` con `resumen.validacion_definitiva`) y los parámetros del tabú (`config/default.json` → `tabu`).
- [ ] **Step 4:** verde. **Step 5:** Commit `feat(cambios): tabla antes/después de la fase 4`.

### Task 2: Escenarios del paso 3 por instancia e invariancia de ω = 0 (Rust)

**Files:** Modify `src/config.rs`, `src/bin/reto.rs` (fn `paso3`), `config/default.json`, `src/paso3.rs` (pruebas), `tests/cli.rs`.

**Interfaces — Produces:** `Config.escenarios_paso3_por_instancia: BTreeMap<String, usize>` (serde default vacío; en `default.json` y `Config::default()`: `{"1ha": 1000, "map_27": 1000}`); `Config::n_escenarios_paso3_para(&self, nombre: &str) -> usize` (valor del mapa o `n_escenarios_paso3`).

- [ ] **Step 1: Pruebas:** en `config.rs` `escenarios_paso3_por_instancia_por_defecto` (1ha → 1000, map_27 → 1000, map_3 → 100); `validar_rechaza_paso3_mayor_que_mc` (valor 1001 con `n_escenarios = 1000` → `Err` que menciona la clave; también `n_escenarios_paso3 > n_escenarios`); `--set escenarios_paso3_por_instancia={}` deja 100 en todas; `defaults_del_paso1_y_json_coinciden` sigue pasando. En `paso3.rs` `omega0_no_depende_de_n`: `correr_instancia` con n = 3 y n = 7 sobre una instancia chica → `filas[0]` (z, m, caso, iteraciones; no `tiempo_s`), φ de ω = 0 y `e0` idénticos, y `filas[..3]` iguales. En `tests/cli.rs`: `paso3` en carpeta temporal con `--set n_escenarios=20 --set n_escenarios_paso3=5 --set 'escenarios_paso3_por_instancia={"1ha":12}'` → `n == 12` para 1ha en `paso3_resumen.csv` (preparar insumos como las pruebas CLI de paso3 existentes).
- [ ] **Step 2:** ver fallar. **Step 3:** implementar; `paso3` usa `config.n_escenarios_paso3_para(nombre)`. **Step 4:** `cargo test` verde.
- [ ] **Step 5:** Commit `feat(paso3): número de escenarios por instancia (1000 en 1ha y P1)`.

### Task 3: Transiciones acumuladas y Z por nivel en el paso 3 (Rust)

**Files:** Create `src/transicion.rs` (registrar en `src/lib.rs`); Modify `src/paso3.rs`, `src/bin/reto.rs` (fn `paso3`), `tests/fase3.rs` o `tests/cli.rs`.

**Interfaces — Produces:**
- `transicion.rs`: `pub struct Conteos { pub vecindad: [[u64; N_ESPECIES]; N_ESPECIES], pub orden: [[u64; N_ESPECIES]; N_ESPECIES], pub plantas: [u64; N_ESPECIES] }` (`plantas` = plantas por especie, nuevas + existentes) con `Default` y `pub fn sumar(&mut self, otro: &Conteos)`; `pub fn orden_serpentina(malla: &Malla, phi: &[u8], existente: &[bool]) -> Vec<usize>`; `pub fn conteos(malla: &Malla, grafo: &Grafo, phi: &[u8], existente: &[bool]) -> Conteos`; `pub fn escribir_transicion(path: &Path, c: &Conteos) -> Result<(), Box<dyn Error>>` → CSV `tipo, de, a, conteo, n, prob` (`tipo ∈ {vecindad, orden}`, 200 filas); `pub fn escribir_plantas(path: &Path, c: &Conteos)` → CSV `especie, plantas, fraccion`.
- `paso3.rs`: `pub struct SalidaPaso3 { pub filas: Vec<FilaPaso3>, pub phi0: Vec<u8>, pub e0: Escenario, pub niveles: Vec<[f64; 6]>, pub conteos: Option<Conteos> }`; `pub fn correr_instancia_detalle(entrada: &EntradaPaso3, n_omega: usize, params: &ParamsTabu, especies: &[Especie], w: &[Vec<f64>], con_transicion: bool) -> SalidaPaso3` — mismo reparto en hilos y semillas que `correr_instancia`; `niveles[ω]` = `practica::desglose(...).niveles` del φ final; `conteos` = suma sobre todos los ω (sólo si `con_transicion`). `correr_instancia` queda igual (puede delegar en la nueva y descartar lo extra).
- `reto paso3`: usa `correr_instancia_detalle` con `con_transicion = nombre ∈ {1ha, map_27}`; escribe `results/paso3_transicion_<inst>.csv` y `results/paso3_plantas_<inst>.csv` (suma sobre todos los escenarios) para esas dos; añade al final de `paso3_escenarios.csv` las columnas `Z_nivel1, Z_nivel2, Z_nivel3, Z_nivel4, Z_nivel5, Z_otro`.

- [ ] **Step 1: Pruebas** en `transicion.rs`, caso fijo compartido con la Task 5: `Malla::rectangular(3, 2, …)` (2 filas × 3 columnas, pos 0..5 por filas), φ = [1, 2, 1, 0, 3, 2] (especies 0,1,0,vacía,2,1), existente en pos 4: `orden_serpentina == [0, 1, 2, 5]` (fila 1 al revés, salta la vacía 3 y la existente 4); `conteos.orden`: especies en ese orden 0, 1, 0, 1 → pares (0→1), (1→0), (0→1) → `orden[0][1] == 2`, `orden[1][0] == 1`, resto 0; `conteos.vecindad` simétrica, sin la vacía, incluye la existente (escribir los valores esperados calculados a mano desde las adyacencias de la malla del caso y dejarlos en la prueba). `escribir_transicion` 200 filas, prob por fila suma 1 o 0. En `paso3.rs`: `detalle_igual_a_correr_instancia` (mismas filas y φ0); `niveles_suman_z` (Σ niveles = z ± 1e-9 en cada ω); `conteos_suman_sobre_escenarios` (con n = 2: conteos = conteos(ω0) + conteos(ω1) calculados aparte). Integración: `paso3` en carpeta temporal escribe `paso3_transicion_1ha.csv` y las 6 columnas nuevas al final.
- [ ] **Step 2–4:** fallar, implementar, `cargo test` verde.
- [ ] **Step 5:** Commits `feat(transicion): conteos por vecindad inmediata y por orden de siembra` y `feat(paso3): transiciones sobre todos los escenarios y Z por nivel por escenario`.

### Task 4: DSATUR por cuotas (Rust) y variantes en la práctica

**Files:** Create `src/dsatur.rs` (registrar en `src/lib.rs`); Modify `src/bin/reto.rs` (fn `practica`), `python/figuras.py` (lista `variantes` de `practica_distribucion`), pruebas en `src/dsatur.rs` y `tests/cli.rs`.

**Interfaces — Produces:** `pub fn dsatur_cuotas(grafo: &Grafo, tabla: &TablaCompetencia, cuotas: &[usize; N_ESPECIES], fijas: &[u8]) -> Vec<u8>` — `fijas[u]` = φ de la existente (0 = libre); devuelve φ completo con las fijas intactas.

Algoritmo (determinista):
1. Mientras queden cuotas > 0: elegir la libre sin asignar con mayor saturación = número de especies distintas entre sus vecinos del anillo 0 ya plantados (fijas o asignadas); empate → más vecinos en `grafo` (todos los anillos); empate → menor índice.
2. Asignarle la especie j con cuota restante > 0 que minimiza Σ_{v vecino plantado} `tabla.peso(anillo, j+1, φv)`; empate → mayor cuota restante; empate → menor j.
3. Las libres sin asignar al agotarse las cuotas quedan vacías.

En `reto practica`, dos filas nuevas por instancia después de las cinco actuales: `dsatur_sin_existentes` (cuotas = D* de `paso1_composicion.csv`, sin fijas) y `dsatur_con_existentes` (fijas = existentes de ω = 0 con su especie; cuotas = n_j de `paso3::cuotas(q, D*, X_ω0, V_lib)`, q de `leer_compra` con ε = 0.1 y π_factor = `config.pi_factor`). `motivo_paro` vacío.

- [ ] **Step 1: Pruebas:** `dsatur_respeta_cuotas`; `dsatur_respeta_cuotas_con_huecos` (Σ cuotas = libres − 3 → exactamente 3 vacías); `dsatur_no_toca_fijas`; `dsatur_determinista`; `dsatur_evita_vecinos_misma_especie_con_tres_especies` (malla 6×6, cuotas 12/12/12 → 0 pares inmediatos de la misma especie; si el algoritmo no lo logra, no relajar: DONE_WITH_CONCERNS con el valor). CLI: `practica` en carpeta temporal escribe 7 variantes por instancia y `dsatur_con_existentes.plantas == tabu_con_existentes.plantas`. `test_figuras` con las variantes nuevas.
- [ ] **Step 2–4:** fallar, implementar, verde.
- [ ] **Step 5:** Commits `feat(dsatur): coloración DSATUR por cuotas` y `feat(practica): variantes DSATUR sin y con existentes`.

### Task 5: Orden de plantación y matrices (`python/transicion.py`)

**Files:** Create `python/transicion.py`, `python/tests/test_transicion.py`; Modify `python/figuras.py` (figura `orden_plantacion`).

**Interfaces — Produces:**
- `orden_serpentina(patron: pd.DataFrame) -> pd.DataFrame` (columnas `paso` 1..M, `pos, fila, col, x, y, especie`; sin columna `existente` → todas las plantadas son nuevas).
- `matriz_orden(orden) -> pd.DataFrame`, `matriz_vecindad(patron, d: float, tol: float = 1e-6) -> pd.DataFrame` en el formato largo de Global Constraints.
- `estacionaria(matriz: pd.DataFrame) -> pd.Series` (índice especie; sólo especies con n > 0; vector propio izquierdo de valor 1 normalizado; las ausentes con 0).
- `main(origen=results, instancias=("1ha","map_27"))` escribe `results/orden_plantacion_<inst>.csv` (de `paso3_patron_<inst>_omega0.csv`) y `results/transicion.csv` con columnas `instancia, patron, alcance, tipo, de, a, conteo, n, prob`: `alcance = todos_los_escenarios` desde `paso3_transicion_<inst>.csv` (principal, `patron = tabu_paso3`), `alcance = omega0` para el ejemplo (`tabu_paso3` y `practica_iii` de `practica_patron_<inst>_iii.csv`). Además `results/transicion_estacionaria.csv`: `instancia, patron, alcance, especie, pi_estacionaria, composicion_plantada` (composición = fracción de plantas por especie, nuevas + existentes: de `paso3_plantas_<inst>.csv` para `todos_los_escenarios`, del patrón para `omega0`).
- Figura `orden_plantacion` (1ha y map_27): la malla con los primeros 60 pasos numerados y unidos por una línea, color por especie, existentes con borde negro.

- [ ] **Step 1: Pruebas:** `test_orden_serpentina_filas_alternas` y `test_orden_salta_existentes_y_vacias` (mismo caso 2×3 de la Task 3 → pos [0, 1, 2, 5]); `test_matriz_orden_conteos` (mismos conteos que la Task 3); `test_matriz_vecindad_igual_que_rust` (mismos conteos que la prueba de vecindad de la Task 3); `test_fila_sin_ocurrencias`; `test_prob_suma_uno`; `test_estacionaria_igual_a_n_relativo` (matriz de vecindad simétrica → π_j = n_j/Σn ± 1e-9); `test_estacionaria_cerca_de_composicion` (malla 20×20, φ aleatorio con semilla fija y composición 10 % / 30 % / 60 % de tres especies → |π − composición| ≤ 0.02 en cada especie); `test_estacionaria_con_especie_ausente`; prueba de `figuras` para `orden_plantacion` con datos sintéticos.
- [ ] **Step 2–4:** fallar, implementar, verde.
- [ ] **Step 5:** Commit `feat(transicion): orden de plantación, matrices agregadas y estacionaria`.

### Task 6: Entorno y tamaño del problema (`python/entorno.py`, `python/tamano.py`)

**Files:** Create `python/entorno.py`, `python/tamano.py`, `python/tests/test_entorno.py`, `python/tests/test_tamano.py`; Modify `python/paso2_compra.py`.

**Interfaces — Produces:**
- `entorno.recolectar(ejecutar=_ejecutar) -> dict` con `modelo_pc, sistema_operativo, disco_total_gb, disco_libre_gb, tipo_disco, ram_gb, ram_modulos (lista de {capacidad_gb, tipo, velocidad_mhz}), procesador, nucleos_fisicos, hilos_logicos, commit, software` (rustc, cargo, python, numpy, scipy, pandas, matplotlib, pulp, highspy, ortools, pytest). Fuentes (PowerShell, `ConvertTo-Json`): `Win32_ComputerSystem` (Manufacturer, Model, TotalPhysicalMemory), `Win32_Processor` (Name, NumberOfCores, NumberOfLogicalProcessors), `Win32_OperatingSystem` (Caption, Version), `Win32_LogicalDisk -Filter "DeviceID='C:'"` (Size, FreeSpace), `Win32_PhysicalMemory` (Capacity, Speed, SMBIOSMemoryType por módulo; tipos 20 DDR, 21 DDR2, 24 DDR3, 26 DDR4, 34 DDR5, 35 LPDDR5, otro → `"código N"`), `Get-PhysicalDisk` (FriendlyName, MediaType); `git rev-parse HEAD`; versiones de Python con `importlib.metadata.version`. `ejecutar(cmd: list[str]) -> str` inyectable. `main()` escribe `results/entorno.json` y `results/entorno.md`.
- `paso2_compra.modelo_ple(C, c, pi_factor, presupuesto=None, eta=0.0) -> tuple[pulp.LpProblem, dict]`; `resolver_ple` lo usa (misma firma y resultados).
- `tamano.tamano_modelos(origen) -> pd.DataFrame` (1ha y map_27; columnas `instancia, modelo, variables, tipo_variables, restricciones, nota`): `paso1_tabu` (|V| enteras en {0..10}); `paso1_lineal` (10·|V| binarias x + 100·|E| y, |E| = Σ `pares_anillo_*` de `malla_resumen.csv`; restricciones |V| + 10 + 20 + 3·100·|E|); `paso2_ple` (conteos reales de `modelo_ple` con los C de ε = 0.1); `paso3_por_escenario` (|V_lib| de ω = 0); `extensivo_completo` (forma extensiva: x, y del paso 1 + q (10) + por cada ω de |Ω| = 1000: x_ω, y_ω del acomodo (10·|V| + 100·|E|) y f_ω, e_ω (20); nota "justifica la descomposición en pasos").
- `tamano.parametros() -> pd.DataFrame` (`parametro, simbolo, cantidad, fuente`): c_j, s1_j, s2_j, δ_j, R_j, rasgos (7×10), W (10×10), θ_j, L_j, U_j, π_j, d, ε, κ, γ*, factor_radios, B, η, μ_ρ, σ_ρ, p̂_j, |Ω|, |Ω₃|, y una última fila `TOTAL` con la suma de `cantidad`.
- `tamano.problema(origen) -> pd.DataFrame` (`instancia, especies, plantas_por_ha, D_tot, n_pos, D_0..D_9, Dnom_0..Dnom_9`) para 1ha y map_27.
- `tamano.main()` escribe `results/tamano_modelo.csv`, `results/tamano_parametros.csv`, `results/tamano_problema.csv`.

- [ ] **Step 1: Pruebas:** `test_recolectar_con_salidas_falsas` (JSON fijos → `nucleos_fisicos == 24`, `hilos_logicos == 32`, `ram_gb` a 1 decimal, dos módulos DDR5 a 5600 MHz, `tipo_disco == "SSD"`, `commit` = el hash falso); `test_recolectar_tolera_comando_fallido` (excepción → `"desconocido"`); `test_modelo_ple_igual_a_resolver`; `test_tamano_paso1_lineal` (|V| = 49 y |E| conocido → 490 + 100·|E|); `test_extensivo_mayor_que_suma` (extensivo > paso1 + paso2 + 1000·paso3); `test_parametros_total` (W 100, rasgos 70, fila TOTAL = suma); `test_problema_tiene_dnom`.
- [ ] **Step 2–4:** fallar, implementar, verde.
- [ ] **Step 5:** Commits `refactor(paso2): modelo_ple separado de resolver_ple`, `feat(entorno): características de la PC, RAM, disco y versiones`, `feat(tamano): tamaño del problema, variables y parámetros`.

### Task 7: Sensibilidades (`python/sensibilidad.py`)

**Files:** Create `python/sensibilidad.py`, `python/tests/test_sensibilidad.py`; Modify `python/figuras.py` (figura `sensibilidad`).

**Interfaces — Produces:**
- `VARIANTES: dict[str, dict]` (las de Global Constraints; `"base": {}`, …, `"s2": {"usar_s2": True}`).
- `config_variante(base: dict, cambios: dict) -> dict`: copia de `config/default.json` con los cambios y `escenarios_paso3_por_instancia = {}`.
- `correr_variante(nombre, carpeta: Path, reto: Path, ejecutar=subprocess.run)`: escribe `carpeta/config.json`; copia `results/mc_params.json`; corre `reto malla`, `reto paso1 --instancias 1ha,map_27,map_21 --solo-gamma-star`, `reto montecarlo --instancias 1ha,map_27`, `paso2_compra.correr(resultados=carpeta)`, `reto paso3 --instancias 1ha,map_27`, `reto practica --instancias 1ha,map_27`, todos con `--config carpeta/config.json --resultados carpeta`. Salta la variante si ya tiene `paso3_resumen.csv` y `practica_distribucion.csv` y su `config.json` es igual.
- `resumir(raiz: Path) -> pd.DataFrame` → `results/sensibilidad.csv`: `variante, instancia, D_0..D_9, suma_theta_d, z_opt_media, z_opt_sd, costo_compra, faltante_esperado, z_paso3_media, z_paso3_sd, z_practica_iii, z_dsatur_con, z_tabu_con, cambio_z_opt_pct, cambio_z_paso3_pct` (vs `base` de la misma instancia; map_21 sólo columnas del paso 1, el resto NaN).
- `main(reto=target/release/reto.exe, raiz=results/sensibilidad)`. Figura `sensibilidad`: barras del cambio % de `z_opt_media` y `z_paso3_media` por variante, una serie por instancia.

- [ ] **Step 1: Pruebas:** `test_config_variante`; `test_config_variante_m0_valida` (skip sin `reto.exe`; `reto paso1 --instancias 1ha --solo-gamma-star --set n_semillas=1 --set 'tabu={"candidatos":16,"tenure":5,"max_iter":200,"max_sin_mejora":50,"prob_intercambio":0.5,"iter_por_posicion":1,"sin_mejora_por_posicion":0}'` con la config `m_0` en `tmp_path` → código 0); `test_resumir` (carpetas sintéticas `base` y `kappa_0.4` → `cambio_z_paso3_pct` correcto; map_21 NaN en paso 3); `test_reanuda_variante_hecha` (`ejecutar` falso no se llama).
- [ ] **Step 2–4:** fallar, implementar, verde. Corrida corta real de `base` con topes chicos en el scratchpad (no se commitea).
- [ ] **Step 5:** Commit `feat(sensibilidad): variantes de κ, radios, m y s2 en 1ha y P1 y paso 1 en map_21`.

### Task 8: Experimentación, RESUMEN, tiempos, run_all.ps1 y README

**Files:** Create `python/experimentacion.py`, `python/tests/test_experimentacion.py`; Modify `python/resumen.py`, `python/tests/test_resumen.py`, `run_all.ps1`, `README.md`.

**Interfaces — Produces:**
- `experimentacion.tabla(origen, instancias=("1ha","map_27")) -> pd.DataFrame` → `results/experimentacion.csv` con columnas `instancia, magnitud, especie, n, media_ha, sd_ha, ic95_inf, ic95_sup`; magnitudes por escenario de `escenarios_X_<inst>.csv` (n = 1000): `T` (total existentes), `X_j`, `C_j = max(D*_j − X_j, 0)`, `n_j = min(q_j, C_j)` (q de `paso2_compra.csv`, ε = 0.1, π_factor del config); `q_j` (constante, sd 0); de `paso3_escenarios.csv` (n del paso 3): `Z`, `Z_nivel1..5`, `Z_otro`. Todo dividido entre `area_ha`; `especie` vacía para T y Z.
- `run_all.ps1`: `Invoke-Paso` mide cada paso con `Measure-Command` y **añade** una fila a `results/tiempos.csv` (`paso, instancias, segundos, fecha`; `instancias` = valor de `--instancias` o vacío; cabecera si no existe); switches nuevos `-OmitirSensibilidad` y `-SoloValidacion` (corre sólo los tres pasos de validación) y parámetro `-ArgumentosValidacion` (arreglo pasado a `validacion_exacta.py`, por defecto vacío). Pasos nuevos después de `practica`: `python\transicion.py`, `python\entorno.py`, `python\tamano.py`, `python\experimentacion.py`, `python\sensibilidad.py` (salvo `-OmitirSensibilidad`); luego `practica_compra`, `markov`, `figuras`, `resumen`.

- [ ] **Step 1: Pruebas** (CSV sintéticos en `tmp_path`): `test_experimentacion_ic` (media, sd e IC con la fórmula de Global Constraints; área 2 ha divide); `test_n_j_es_min_q_c`; en `test_resumen.py`: `## Entorno y software` (modelo, RAM con tipo y velocidad, disco con tipo, núcleos, commit, y la tabla de `tiempos.csv` con la última corrida de cada paso; mensaje si falta cada archivo); `## Tamaño del problema` (las tres tablas, con la fila TOTAL y la forma extensiva); `## Experimentación (1000 simulaciones)` (de `experimentacion.csv`: por especie y por ha, E[X_j], E[C_j], q_j, E[n_j] con IC, E[T] y E[Z] y E[Z por nivel] con IC; nota de que las otras 29 instancias usan 100 escenarios del paso 3); `## Orden de plantación y transiciones` (primeros 21 pasos de `orden_plantacion_1ha.csv`; las dos matrices agregadas de 1ha en formato ancho con 3 decimales como principales; la estacionaria contra la composición; la de ω = 0 sólo como ejemplo); DSATUR en `## Práctica actual` (una frase comparando su Z con tabú y serpentina); `## Sensibilidades` (tabla de `sensibilidad.csv`). `_pendiente` deja de listar DSATUR y sensibilidades cuando existen sus CSV. Figuras nuevas en `FIGURAS_LISTA`.
- [ ] **Step 2–4:** fallar, implementar, verde. `run_all.ps1` validado con `[ScriptBlock]::Create` y con una prueba de `Invoke-Paso` sobre un comando trivial (`cmd /c exit 0`) que escriba la fila de tiempos en una carpeta temporal (parámetro `-Resultados` o variable de entorno para la ruta de `tiempos.csv`).
- [ ] **Step 5:** README con comandos y salidas nuevas. Commits `feat(experimentacion): valores esperados por ha con IC 95 %`, `feat(resumen): secciones de la entrega final`, `feat(run_all): tiempos por paso y switches de sensibilidad y validación`.

### Task 9: Corridas finales y resultados

Sólo el controlador; sin código nuevo salvo correcciones con prueba.

- [ ] `cargo build --release`.
- [ ] **Comprobación de ω = 0 (antes de tocar `results/`):** copiar a un directorio del scratchpad los insumos del paso 3 (`mc_params.json`, `paso1_composicion.csv`, `paso1_patron_{1ha,map_27}.csv`, `paso2_compra.csv`) y correr `reto paso3 --instancias 1ha,map_27 --resultados <dir>` (1000 escenarios). Comparar con `results/`: `paso3_patron_<inst>_omega0.csv` idéntico byte a byte y la fila ω = 0 de `paso3_escenarios.csv` idéntica salvo `tiempo_s` y las columnas nuevas. **Si difiere, detenerse y avisar al usuario.** En el mismo directorio, con `--set 'escenarios_paso3_por_instancia={"1ha":1}'`, comparar `paso3_transicion_1ha.csv` contra `transicion.matriz_vecindad`/`matriz_orden` de Python sobre el patrón ω = 0: conteos idénticos.
- [ ] `.\run_all.ps1 -OmitirValidacion` completo (con sensibilidades; los pasos ya hechos dan los mismos resultados por las semillas fijas) → `results/tiempos.csv`.
- [ ] `.\run_all.ps1 -SoloValidacion -ArgumentosValidacion '--formulaciones','ajustada','--workers','32','--actualizar'` (≈ 1 h 40 min; nada más corriendo), luego `python\resumen.py`.
- [ ] `python\cambios.py --despues 45c370f`.
- [ ] Revisar que RESUMEN tenga cada punto de la adenda; commits de resultados por bloque (`chore(results): …`).
- [ ] Revisión final de la rama (modelo más capaz); avisar al usuario con la tabla de cambios, la de validación y los números de la entrega. Sin push ni merge.
