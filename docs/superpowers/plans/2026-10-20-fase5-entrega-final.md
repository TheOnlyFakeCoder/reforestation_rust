# Fase 5 — Entrega final (experimentación y resultados): plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (o executing-plans). TDD (prueba primero, verla fallar, implementar). Commits pequeños en español, **sin línea de atribución** (`Claude-Session` prohibido). Nunca `git push` ni merge.

**Goal:** Producir todo lo que pide la entrega final (entorno, tamaño del problema, 1000 simulaciones, competencia esperada, matrices de transición, orden de plantación) más lo que se había recortado (DSATUR, sensibilidades), y cerrar la fase 4 (tabla de cambios, validación con 32 workers), con una sola tanda de corridas largas al final.

**Architecture:** Rust añade un módulo `dsatur.rs` (variantes nuevas en `reto practica`) y escenarios del paso 3 por instancia en el config. Python añade scripts pequeños e independientes que leen `results/` (`cambios.py`, `transicion.py`, `entorno.py`, `tamano.py`, `sensibilidad.py`) y extiende `figuras.py`/`resumen.py`. `sensibilidad.py` orquesta `reto.exe` con `--config`/`--resultados` en `results/sensibilidad/<variante>/`.

**Tech Stack:** Rust estable (csv, serde_json), Python `.venv` (numpy, pandas, matplotlib, pulp/highspy, ortools, pytest).

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` (§2 W, §3 alcance, §5 paso 1, §7 paso 3, §10 práctica y DSATUR, §11 Markov) y la adenda `docs/superpowers/specs/2026-10-20-fase5-entrega-final.md` (requisitos de la entrega y decisiones del usuario). Ledger de la fase 4: `.superpowers/sdd/2026-10-18-fase4-convergencia-validacion/progress.md`.

## Global Constraints

- No borrar archivos, funciones ni columnas existentes; columnas nuevas al final; variantes nuevas como filas nuevas. Lo reemplazado queda marcado LEGACY o al lado.
- `cargo test` normal (sin reporter ni `RUSTC_BOOTSTRAP`) y `.venv\Scripts\python.exe -m pytest -q python/tests` en verde en cada commit.
- Las pruebas no escriben en `results/` (usan `tmp_path` / directorios temporales).
- Ninguna tarea de implementación (1–7) corre pasos largos sobre `results/`; las corridas sobre `results/` sólo en la Task 8. Las corridas cortas de prueba van a un directorio temporal o al scratchpad.
- Codificación: φ interno 0 = vacía, especie j ↔ j+1; CSV con `especie` 0..9 y −1 vacía.
- Instancias: `1ha` y `map_27` (= P1); `map_21` = polígono de 8 ha para la comprobación de sensibilidad.
- Paso 3: 1000 escenarios en `1ha` y `map_27`, 100 en las demás (decisión del usuario).
- Sensibilidades: variantes exactas `base`, `kappa_0.4` (`kappa=0.4`), `kappa_0.8`, `radios_0.75` (`factor_radios=0.75`), `radios_1.25`, `m_0` (`m_escala=0`), `m_0.5`, `m_2`, `s2` (`usar_s2=true`); cadena completa en 1ha y map_27 con **100** escenarios del paso 3 en todas las variantes (incluida `base`, para comparar en igualdad de condiciones); sólo paso 1 γ* en map_21.
- Base "antes" de la tabla de cambios de la fase 4: commit `be65ce0`; "después": el commit con los resultados de la re-corrida de la fase 4 (lo hace el controlador antes de la Task 1; se pasa como argumento).

## Review Focus

- `m_escala = 0` → θ = 0: la config debe aceptarlo y el paso 1 se reduce a minimizar Z; la sensibilidad no debe fallar (Task 6, prueba `test_config_variante_m0_valida` que corre `reto paso1` con `--set m_escala=0` sobre una carpeta temporal).
- DSATUR con Σ cuotas < posiciones libres (caso presupuesto corto del paso 3): deben quedar vacías exactamente las sobrantes y las cuotas cumplirse (Task 3, `dsatur_respeta_cuotas_con_huecos`).
- `escenarios_paso3_por_instancia` con un valor > `n_escenarios` (más escenarios que los generados por Monte Carlo) → error claro de validación, no lectura fuera de rango (Task 2, `validar_rechaza_paso3_mayor_que_mc`).
- Orden de siembra con existentes y vacías: el recorrido salta ambas; la matriz por orden cuenta sólo pares consecutivos de plantas sembradas (Task 4, `test_orden_salta_existentes_y_vacias`).
- Filas de especies sin ninguna ocurrencia en la matriz de transición: probabilidad NaN no; se reporta la fila con conteo 0 y probabilidades 0, y la columna `n` lo deja claro (Task 4, `test_fila_sin_ocurrencias`).

---

### Task 1: Tabla de cambios de la fase 4 (`python/cambios.py`)

**Files:** Create `python/cambios.py`, `python/tests/test_cambios.py`.

**Interfaces — Produces:** `leer_git(commit: str, ruta: str) -> pd.DataFrame` (lee `git show <commit>:<ruta>` como CSV); `tabla_cambios(antes: dict[str, pd.DataFrame], despues: dict[str, pd.DataFrame]) -> pd.DataFrame` con columnas `cifra, instancia, antes, despues, cambio_pct`; `main(antes="be65ce0", despues="HEAD", salida="results/cambios_fase4.md")`. Los dicts tienen claves `paso1_gamma_star`, `paso1_composicion`, `paso3_resumen`, `paso2_compra`, `practica_distribucion`.

- [ ] **Step 1: Pruebas** en `test_cambios.py` con DataFrames sintéticos mínimos:
  - `test_tabla_cambios_z_opt`: z_opt de γ* = 3 (media sobre semillas) 100 → 99 en `1ha` da fila `cifra == "Z_opt paso 1"`, `cambio_pct == -1.0`.
  - `test_tabla_cambios_reduccion_serpentina`: Z `iii_serpentina_existentes` 200 y `tabu_con_existentes` 150 → fila "reducción vs serpentina iii" `despues == 25.0` (en %).
  - `test_tabla_cambios_compra`: fila "costo paso 2" sólo con ε = 0.1, π_factor = 2.
  - `test_d_star_sin_cambio`: D* igual antes/después → fila "D*" con `antes == despues` como texto `"[…]"`.
- [ ] **Step 2:** correr y ver fallar.
- [ ] **Step 3: Implementar.** Cifras e instancias: Z_opt paso 1 (1ha, map_27, map_21); D* (mismas); Z media paso 3 (1ha, map_27, map_21); costo y q del paso 2 (1ha, map_27; ε = 0.1, π_factor = 2); reducción % de Z de `tabu_con_existentes` frente a `iii_serpentina_existentes` (1ha, map_27). `main` escribe Markdown: la tabla, la tabla de validación (filas definitivas de `results/validacion_exacta.csv`, reutilizando `resumen.validacion_definitiva`) y los parámetros finales del tabú (`config/default.json` → `tabu`). CLI `--antes`, `--despues`, `--salida`.
- [ ] **Step 4:** pytest verde.
- [ ] **Step 5:** Commit `feat(cambios): tabla antes/después de la fase 4`. (El controlador corre `main` y commitea `results/cambios_fase4.md` aparte.)

### Task 2: Escenarios del paso 3 por instancia (Rust)

**Files:** Modify `src/config.rs`, `src/bin/reto.rs` (fn `paso3`), `config/default.json`, `tests/cli.rs`.

**Interfaces — Produces:** `Config.escenarios_paso3_por_instancia: BTreeMap<String, usize>` (serde default vacío; en `default.json` y en `Config::default()`: `{"1ha": 1000, "map_27": 1000}`); `Config::n_escenarios_paso3_para(&self, nombre: &str) -> usize` (valor del mapa o `n_escenarios_paso3`).

- [ ] **Step 1: Pruebas** en `config.rs`: `escenarios_paso3_por_instancia_por_defecto` (`n_escenarios_paso3_para("1ha") == 1000`, `("map_27") == 1000`, `("map_3") == 100`); `validar_rechaza_paso3_mayor_que_mc` (valor 1001 con `n_escenarios = 1000` → `Err` que menciona `escenarios_paso3_por_instancia`; también `n_escenarios_paso3 > n_escenarios`); `--set escenarios_paso3_por_instancia={}` deja 100 en todas. La prueba existente `defaults_del_paso1_y_json_coinciden` debe seguir pasando (Rust default = JSON). En `tests/cli.rs`: `paso3` sobre carpeta temporal con `--set n_escenarios=20 --set n_escenarios_paso3=5 --set 'escenarios_paso3_por_instancia={"1ha":12}'` escribe `n == 12` en `paso3_resumen.csv` para 1ha (seguir el patrón de las pruebas CLI de paso3 existentes para preparar los insumos).
- [ ] **Step 2:** ver fallar. **Step 3:** implementar; `paso3` llama `correr_instancia(&entrada, config.n_escenarios_paso3_para(nombre), &params)`. **Step 4:** `cargo test` verde.
- [ ] **Step 5:** Commit `feat(paso3): número de escenarios por instancia (1000 en 1ha y P1)`.

### Task 3: DSATUR por cuotas (Rust) y variantes en la práctica

**Files:** Create `src/dsatur.rs` (registrar en `src/lib.rs`); Modify `src/bin/reto.rs` (fn `practica`), `python/figuras.py` (lista `variantes` de la figura `practica_distribucion`), `tests/fase3.rs` o `tests/cli.rs`.

**Interfaces — Produces:**
`pub fn dsatur_cuotas(grafo: &Grafo, tabla: &TablaCompetencia, cuotas: &[usize; N_ESPECIES], fijas: &[u8]) -> Vec<u8>` — `fijas[u]` = φ de la existente (0 = libre); devuelve φ completo (fijas intactas).

Algoritmo (determinista):
1. Libres = posiciones con `fijas[u] == 0`. Mientras queden cuotas > 0: elegir la libre sin asignar con mayor saturación = número de especies distintas entre sus vecinos **inmediatos** (anillo 0) ya plantados (fijas o asignadas); empate → mayor número de vecinos en `grafo` (todos los anillos); empate → menor índice.
2. Asignarle la especie j con cuota restante > 0 que minimiza Σ_{v vecino plantado} `tabla.peso(anillo, j+1, φv)`; empate → mayor cuota restante; empate → menor j.
3. Las libres que quedan sin asignar al agotarse las cuotas quedan vacías (0).

En `reto practica` dos filas nuevas por instancia, después de las cinco existentes: `dsatur_sin_existentes` (cuotas = D* de `paso1_composicion.csv`, sin fijas) y `dsatur_con_existentes` (fijas = existentes de ω = 0 con su especie, cuotas = n_j de `paso3::cuotas(q, D*, X_ω0, V_lib)` con q de `leer_compra` ε = 0.1, π_factor = `config.pi_factor`). `motivo_paro` vacío. En `figuras.py`, añadir las dos variantes a la lista de la figura de barras.

- [ ] **Step 1: Pruebas** unitarias en `dsatur.rs` sobre `Malla::rectangular` chica y `TablaCompetencia` real (como en `competencia.rs` tests): `dsatur_respeta_cuotas` (conteo por especie = cuotas, Σ = |V|); `dsatur_respeta_cuotas_con_huecos` (Σ cuotas = |libres| − 3 → exactamente 3 vacías, conteos exactos); `dsatur_no_toca_fijas`; `dsatur_determinista` (dos llamadas iguales); `dsatur_evita_vecinos_misma_especie_con_tres_especies`: malla 6×6, tres especies con cuotas 12/12/12 y las demás 0 → 0 pares inmediatos de la misma especie (la malla tresbolillo es 3-coloreable con clases iguales). Si el algoritmo descrito no lo logra, no relajar la prueba: reportarlo como DONE_WITH_CONCERNS con el valor obtenido. Prueba CLI/integración: `practica` en carpeta temporal escribe 7 variantes por instancia, con `dsatur_con_existentes.plantas == tabu_con_existentes.plantas`.
- [ ] **Step 2–4:** fallar, implementar, `cargo test` y pytest (`test_figuras`) verdes.
- [ ] **Step 5:** Commits `feat(dsatur): coloración DSATUR por cuotas` y `feat(practica): variantes DSATUR sin y con existentes`.

### Task 4: Orden de plantación y matrices de transición (`python/transicion.py`)

**Files:** Create `python/transicion.py`, `python/tests/test_transicion.py`; Modify `python/figuras.py` (figura `orden_plantacion`).

**Interfaces — Produces:**
- `orden_serpentina(patron: pd.DataFrame) -> pd.DataFrame`: filas ordenadas por `fila`; dentro de la fila, `col` creciente en filas pares y decreciente en impares; sólo plantas a sembrar (`especie >= 0` y `existente == 0`; si no hay columna `existente`, todas las plantadas). Columnas de salida: `paso` (1..M), `pos, fila, col, x, y, especie`.
- `matriz_orden(orden: pd.DataFrame) -> pd.DataFrame` y `matriz_vecindad(patron: pd.DataFrame, d: float, tol: float = 1e-6) -> pd.DataFrame`: formato largo `de, a, conteo, n, prob` para j, k ∈ 0..9 (100 filas); `n` = Σ conteos de la fila; `prob = conteo / n` o 0 si `n == 0`. Por orden: pares (paso t, paso t+1). Por vecindad: pares ordenados (u, v) de plantas (sembradas **o existentes**) a distancia ≤ d + tol, contando ambos sentidos.
- `main(origen=results, instancias=("1ha","map_27"))` escribe `results/orden_plantacion_<inst>.csv` (desde `paso3_patron_<inst>_omega0.csv`) y `results/transicion.csv` con columnas `instancia, patron, tipo, de, a, conteo, n, prob`, `patron ∈ {tabu_paso3, practica_iii}` (de `practica_patron_<inst>_iii.csv`), `tipo ∈ {orden, vecindad}`. `d` de `config/default.json`.
- Figura `orden_plantacion` (por instancia, 1ha y map_27): la malla con los primeros 60 pasos numerados y unidos por una línea, color por especie, existentes con borde negro.

- [ ] **Step 1: Pruebas:** `test_orden_serpentina_filas_alternas` (malla 2×3 → pasos en orden pos 0,1,2,5,4,3); `test_orden_salta_existentes_y_vacias`; `test_matriz_orden_conteos` (secuencia 0,0,1 → conteo(0→0)=1, (0→1)=1, prob fila 0 = 0.5/0.5); `test_fila_sin_ocurrencias` (fila de especie ausente: n = 0, prob 0); `test_matriz_vecindad_simetrica` (conteo(j→k) == conteo(k→j)); `test_prob_suma_uno` (cada fila con n > 0 suma 1 ± 1e-12).
- [ ] **Step 2–4:** fallar, implementar, verde (incluida una prueba de `figuras` que genere `orden_plantacion` con datos sintéticos en `tmp_path`).
- [ ] **Step 5:** Commit `feat(transicion): orden de plantación y matrices de transición por orden y por vecindad`.

### Task 5: Entorno y tamaño del problema (`python/entorno.py`, `python/tamano.py`)

**Files:** Create `python/entorno.py`, `python/tamano.py`, `python/tests/test_entorno.py`, `python/tests/test_tamano.py`; Modify `python/paso2_compra.py` (separar la construcción del modelo).

**Interfaces — Produces:**
- `entorno.recolectar(ejecutar=subprocess_run) -> dict` con claves `modelo_pc, sistema_operativo, disco_total_gb, disco_libre_gb, ram_gb, procesador, nucleos_fisicos, hilos_logicos, software` (dict nombre → versión: rustc, cargo, python, numpy, scipy, pandas, matplotlib, pulp, highspy, ortools, pytest). En Windows usa `powershell -NoProfile -Command "Get-CimInstance Win32_ComputerSystem | ConvertTo-Json"` (modelo, RAM), `Win32_Processor` (nombre, NumberOfCores, NumberOfLogicalProcessors), `Win32_OperatingSystem` (Caption, Version), `Win32_LogicalDisk -Filter "DeviceID='C:'"` (Size, FreeSpace); versiones de Python con `importlib.metadata.version`. `ejecutar(cmd) -> str` es inyectable para las pruebas. `main()` escribe `results/entorno.json` y `results/entorno.md`.
- `paso2_compra.modelo_ple(C, c, pi_factor, presupuesto=None, eta=0.0) -> (pulp.LpProblem, dict_q)`; `resolver_ple` lo usa (misma firma y resultados que hoy).
- `tamano.tamano_modelos(origen) -> pd.DataFrame` para 1ha y map_27, columnas `instancia, modelo, variables, tipo_variables, restricciones, nota`: paso 1 tabú (φ: |V| enteras en {0..10}); paso 1 lineal equivalente (x: 10·|V| binarias + y por arista inmediata/anillos × 100 pares; restricciones de asignación |V|, composición 10, banda 20); paso 2 PLE (conteo real `len(prob.variables())`, `len(prob.constraints)` de `modelo_ple` con los C de ε = 0.1); paso 3 por escenario (|V_lib| de ω = 0 de `paso3_escenarios.csv`). Aristas desde `malla_resumen.csv` (`pares_anillo_*`).
- `tamano.parametros() -> pd.DataFrame` columnas `parametro, simbolo, cantidad, fuente`: c_j, s1_j, s2_j, δ_j (densidad), R_j, rasgos (7×10), W (10×10), θ_j, L_j, U_j, π_j, d, ε, κ, γ*, factor_radios, B, η, μ_ρ, σ_ρ, p̂_j, |Ω|, |Ω₃|. Escribe `results/tamano_modelo.csv`, `results/tamano_parametros.csv`, más una tabla `results/tamano_problema.csv` (`instancia, especies, plantas_por_ha, D_tot, n_pos, D_0..D_9`).

- [ ] **Step 1: Pruebas:** `test_recolectar_con_salidas_falsas` (ejecutar falso devuelve JSON fijos → `nucleos_fisicos == 24`, `hilos_logicos == 32`, `ram_gb` redondeado a 1 decimal); `test_recolectar_tolera_comando_fallido` (ejecutar lanza → campo `"desconocido"`, no excepción); `test_modelo_ple_igual_a_resolver` (mismo q que antes en un caso chico, el que ya usa `test_paso2.py`); `test_tamano_paso1_lineal` (|V| = 49, aristas conocidas → variables = 490 + 100·aristas); `test_parametros_total` (W cuenta 100, rasgos 70).
- [ ] **Step 2–4:** fallar, implementar, verde.
- [ ] **Step 5:** Commits `refactor(paso2): modelo_ple separado de resolver_ple`, `feat(entorno): características de la PC y versiones`, `feat(tamano): tamaño del problema, variables y parámetros`.

### Task 6: Sensibilidades (`python/sensibilidad.py`)

**Files:** Create `python/sensibilidad.py`, `python/tests/test_sensibilidad.py`; Modify `python/figuras.py` (figura `sensibilidad`).

**Interfaces — Produces:**
- `VARIANTES: dict[str, dict]` exactamente las de Global Constraints (`"base": {}`, `"kappa_0.4": {"kappa": 0.4}`, …, `"s2": {"usar_s2": True}`).
- `config_variante(base: dict, cambios: dict) -> dict`: copia de `config/default.json` con los cambios y `escenarios_paso3_por_instancia = {}` (100 escenarios en todas).
- `correr_variante(nombre, carpeta: Path, reto: Path, instancias=("1ha","map_27"), extra_paso1=("map_21",))`: escribe `carpeta/config.json`; copia `results/mc_params.json`; corre en orden `reto malla`, `reto paso1 --instancias 1ha,map_27,map_21 --solo-gamma-star`, `reto montecarlo --instancias 1ha,map_27`, `paso2_compra.correr(resultados=carpeta)`, `reto paso3 --instancias 1ha,map_27`, `reto practica --instancias 1ha,map_27`, todos con `--config carpeta/config.json --resultados carpeta`. Si una variante ya tiene `paso3_resumen.csv` y `practica_distribucion.csv` y su `config.json` es igual, la salta (reanudar).
- `resumir(raiz: Path) -> pd.DataFrame` → `results/sensibilidad.csv` columnas `variante, instancia, D_0..D_9, suma_theta_d, z_opt_media, z_opt_sd, costo_compra, faltante_esperado, z_paso3_media, z_paso3_sd, z_practica_iii, z_dsatur_con, z_tabu_con, cambio_z_paso3_pct` (vs `base` de la misma instancia; map_21 sólo columnas del paso 1).
- `main(reto=target/release/reto.exe, raiz=results/sensibilidad)`.
- Figura `sensibilidad` (no por instancia): barras del cambio % de `z_opt_media` y `z_paso3_media` vs base por variante, una serie por instancia.

- [ ] **Step 1: Pruebas:** `test_config_variante` (κ 0.4 aplicado, mapa de escenarios vacío, resto igual a default); `test_config_variante_m0_valida` (skip si no existe `reto.exe`; `reto paso1 --instancias 1ha --solo-gamma-star --set n_semillas=1 --set 'tabu={"candidatos":16,"tenure":5,"max_iter":200,"max_sin_mejora":50,"prob_intercambio":0.5,"iter_por_posicion":1,"sin_mejora_por_posicion":0}'` con la config `m_0` en `tmp_path` termina con código 0); `test_resumir` con dos carpetas sintéticas (`base`, `kappa_0.4`) y CSV mínimos → `cambio_z_paso3_pct` correcto y map_21 con NaN en columnas del paso 3; `test_reanuda_variante_hecha` (con `ejecutar` falso, no se llama si ya está hecha).
- [ ] **Step 2–4:** fallar, implementar, verde. Una corrida corta real de `base` con topes chicos en el scratchpad para comprobar la cadena (no se commitea).
- [ ] **Step 5:** Commit `feat(sensibilidad): variantes de κ, radios, m y s2 en 1ha y P1 y paso 1 en map_21`.

### Task 7: RESUMEN, run_all.ps1 y README

**Files:** Modify `python/resumen.py`, `python/tests/test_resumen.py`, `run_all.ps1`, `README.md`.

- [ ] **Step 1: Pruebas** en `test_resumen.py` (CSV sintéticos en `tmp_path`): sección `## Entorno y software` con modelo, RAM y núcleos (o "no se ha corrido `python/entorno.py`" si falta); `## Tamaño del problema` (tablas de `tamano_*.csv`); `## Experimentación (1000 simulaciones)` con, para 1ha y P1 por ha: E[T]/ha (media de `T` de `escenarios_X_<inst>.csv` / área), E[X_j]/ha, E[C_j]/ha (`mc_resumen_especies.csv`, `E_C`/área), q_j/ha del paso 2 (ε = 0.1, π = 2) y E[Z] del paso 3 con IC 95 % (`Z_media ± 1.96·Z_sd/√n`) y n; nota de que las otras 29 instancias usan 100 escenarios con su IC; `## Orden de plantación y transiciones` con los primeros 21 pasos de `orden_plantacion_1ha.csv` y las dos matrices 10×10 de 1ha (tabú) en formato ancho, prob con 3 decimales; DSATUR dentro de `## Práctica actual` (las filas aparecen por venir de `practica_distribucion.csv`; añadir una frase comparando Z de DSATUR con tabú y serpentina); `## Sensibilidades` (tabla de `sensibilidad.csv`). `_pendiente` deja de listar DSATUR y las sensibilidades cuando existen sus CSV (como ya hace con CP-SAT). Figuras nuevas en `FIGURAS_LISTA`.
- [ ] **Step 2–4:** fallar, implementar, verde.
- [ ] **Step 5:** `run_all.ps1`: después de `practica` añadir `python\transicion.py`, `python\entorno.py`, `python\tamano.py`; switch nuevo `-OmitirSensibilidad` que salta `python\sensibilidad.py`; luego `practica_compra`, `markov`, `figuras`, `resumen` como hoy. Validar con el parseo de PowerShell (`[ScriptBlock]::Create`). README: comandos nuevos y salidas. Commit `feat(resumen): secciones de la entrega final; run_all y README`.

### Task 8: Corridas finales y resultados

Sólo el controlador; sin código nuevo salvo correcciones con prueba.

- [ ] `cargo build --release`; `reto paso3 --instancias todas` (1000 escenarios en 1ha y map_27); `reto practica --instancias todas` (DSATUR en las 31).
- [ ] `python\transicion.py`, `python\entorno.py`, `python\tamano.py`, `python\sensibilidad.py`, `python\practica_compra.py`, `python\markov.py`.
- [ ] Validación: `python\validacion_exacta.py --formulaciones ajustada --workers 32 --actualizar` (≈ 1 h 40 min; nada más corriendo en paralelo).
- [ ] `python\figuras.py`, `python\resumen.py`, `python\cambios.py --despues <commit de la re-corrida de la fase 4>`.
- [ ] Revisar que RESUMEN tenga cada punto de la adenda; commits de resultados por bloque (`chore(results): …`).
- [ ] Revisión final de la rama (modelo más capaz); avisar al usuario con la tabla de cambios, la de validación y los números de la entrega. Sin push ni merge.
