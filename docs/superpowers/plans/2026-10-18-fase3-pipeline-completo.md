# Fase 3 — Pipeline completo del modelo: plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. TDD (prueba primero, verla fallar, implementar) por función, sin tdd-guard. Commits pequeños en español, **sin línea de atribución** (`Claude-Session` prohibido). Nunca `git push` ni merge.

**Goal:** Del dato al reporte en un día: terminar paso 1 (barrido γ), ajuste de distribuciones,
Monte Carlo, paso 2 (compra), paso 3 (ajuste al polígono real), comparación con la práctica
actual, Markov, figuras, `results/RESUMEN.md` y README. Prioridad: lo que es parte del modelo y sus
tablas/figuras; validación CP-SAT, DSATUR y sensibilidades extra al final si hay tiempo.

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` (manda). Decisiones del usuario
posteriores: malla rectangular con D_tot = round(658·A) (fase 1b); γ* = 1 provisional hasta que el
usuario vea la curva.

**Repo:** Rust (`src/`, binario `reto` en `src/bin/reto.rs`), Python en `python/` con `.venv`
(`.venv/Scripts/python.exe`; numpy, scipy, pandas, matplotlib, pulp, highspy, ortools, pytest).
Windows; usar Bash con rutas entre comillas. `cargo test` normal debe pasar siempre.

## Estado de partida (lo que ya existe)

- `datos.rs` (`Datos::base()/cargar()`, `Especie{nombre,delta,c,s1,s2,...}`, `Poligono{id,map_id,area,existentes}`), `config.rs` (`Config{d,epsilon,kappa,factor_radios,tolerancia}`, `#[serde(default, deny_unknown_fields)]`, `leer`, `cargar`, `validar`, `aplicar_todas`), `malla.rs` (`Malla::para_objetivo(objetivo,d)`, `filas()`, `fila(f)`, `vecindad(radio,tol)`, `Vecino{pos:u32,anillo:u16}`, `Posicion{fila,col,x,y}`), `composicion.rs` (`composicion_nominal`, `banda`, `Banda{l,u}`), `competencia.rs` (`matriz_w`, `radios`, `radio_max`, `alcance`, `TablaCompetencia::nueva/peso(anillo,a,b)`, códigos φ: 0 vacía, especie j ↔ j+1), `objetivo.rs` (`Grafo::desde`, `z_total`, `delta_reasignacion`, `delta_intercambio`), `busqueda_tabu.rs` (`Problema{grafo,tabla,fija,gamma,theta,restriccion}`, `Restriccion::{Banda{l,u},ComposicionFija}`, `ParamsTabu{candidatos,tenure,max_iter,max_sin_mejora,prob_intercambio,semilla}`, `buscar() -> Resultado{phi,z,objetivo,iteraciones,traza,curva,tiempo_s}`), `paso1.rs` (en curso, ver Task 1), `reportes.rs`, CLI `reto datos|malla` con `--datos --resultados --config --set`.
- `tabu::DEFAULT_ROW_SEQUENCE` (21 índices de especie 0..9) es la secuencia de la socia.
- Pruebas: `tests/fase1.rs`, `tests/fase2.rs`, `tests/cli.rs`, más unitarias en módulos.

## Global Constraints

- Instancias: `"1ha"` (área 1, map_id 0 para semillas) y `"map_<map_id>"` para los 30 polígonos. Reportes usan map_id.
- D_tot = round(658·A); malla `Malla::para_objetivo(D_tot, d)`; excedente = posiciones vacías.
- θ_j = m_escala·c_j·(1 − s_j), s = s1 (s2 si `usar_s2`). Paso 1: min γ·Z + Σθ·D en banda; c_j NO entra.
- Ajuste C: tras el paso 1, búsqueda sólo con intercambios (γ = 1, θ = 0, `ComposicionFija`) → Z_opt(D*).
- Escenarios: **Rust es la única fuente**. Semilla del escenario ω de la instancia con map_id m: `ChaCha8Rng::seed_from_u64(semilla ^ (m as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (omega as u64).wrapping_mul(0xD1B5_4A32_D192_ED03))`. Orden de sorteo fijo: ρ ~ Normal(μ, σ) → T = round(ρ·A) → X ~ Multinomial(T, p̂) (T sorteos categóricos) → `rand::seq::index::sample(rng, |V|, T)` posiciones, asignadas en orden a las especies (X_0 primeras, luego X_1, …).
- μ, σ (MLE) y p̂ los escribe Python en `results/mc_params.json` `{"mu":…, "sigma":…, "p_hat":[10]}`; Rust lo lee.
- C_j = D_j − X_j; para el paso 3 se usa C⁺ = max(C, 0) y se cuentan escenarios/especies con C < 0.
- Paso 2: min Σ c q + (1/|Ω|) Σ_ω Σ_j (π_j f + η_j e); f ≥ C − q, e ≥ q − C, f,e ≥ 0; Σ c q ≤ B·A; q entero ≥ 0. π = 2c (sens. 4c, 10c), η = 0. B = 12,500 $/ha con el área real. Voceador sin presupuesto: q* = cuantil (π−c)/(π+η) de C.
- Paso 3: n_j = min(q_j, C⁺_jω); M = min(|V_lib|, Σ n_j); min Z con existentes fijas; casos `usual` (Σn = |V_lib|), `presupuesto_corto` (Σn < |V_lib|), `existentes_de_sobra` (Σn > |V_lib|).
- Niveles de competencia (práctica): 1 misma especie; 2 mismo género (j≠k); 3 0.2 ≤ w ≤ 0.5 (no niveles 1–2); 4 Yucca–Prosopis; 5 Prosopis–suculentas (especies 0..7); resto "otro".
- CSV con encabezado; floats con `to_string()` (Rust) o 6 decimales (Python). Figuras PNG 300 dpi en `figures/`.
- Commits sin `Claude-Session`. No modificar la lógica de módulos legacy.

---

### Task 1: Paso 1 completo, configuración y `reto paso1`

**Files:** `src/paso1.rs` (terminar), `src/config.rs`, `config/default.json`, `src/bin/reto.rs`, `src/reportes.rs` (o nuevo módulo de salida), `tests/fase2.rs`, `tests/cli.rs`.

Estado: `paso1.rs` tiene `theta`, `patron_serpentina`, `reparar_a_banda`, `Instancia{nombre,malla,d_tot,banda,grafo,tabla}` y `resolver_paso1` a medio cablear (**no compila**: falta `use crate::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};`, el parámetro se llama `_gamma` y se usa `gamma`, falta el campo `theta: [f64; N_ESPECIES]` en `Instancia`, que se llena con `theta(&datos.especies, m_escala, usar_s2)`).

1. Arreglar la compilación y que pasen `resolver_paso1_reoptimiza_z_con_la_composicion_fija` y `resolver_paso1_mejora_la_serpentina_de_la_socia` (tests/fase2.rs).
2. `FilaPaso1` completa: `instancia, map_id: u32, gamma, semilla, d: [usize;10], d_tot, suma_theta_d, suma_c_d, z_paso1, z_opt, z_opt_por_planta (= z_opt/d_tot), tiempo_paso1_s, iter_paso1, tiempo_opt_s, iter_opt`. `Instancia` guarda también `costos: [f64;10]` y `map_id`.
3. `barrido_gamma(instancia, gammas, semillas, params) -> Vec<FilaPaso1>`; extensión: si D* de la mejor semilla (menor γ·z_paso1+Σθ·D… usar el objetivo del paso 1) cambia entre los dos γ mayores, agregar γ_max·3, hasta 3 veces.
4. `Config` agrega (con defaults y en `config/default.json`): `gammas: [0,0.1,0.3,1,3,10,30,100]`, `gamma_star: 1.0`, `n_semillas: 3`, `semilla: 20261017`, `m_escala: 1.0`, `usar_s2: false`, `presupuesto_ha: 12500.0`, `n_escenarios: 1000`, `n_escenarios_paso3: 100`, `pi_factor: 2.0`, y `tabu: {candidatos: 64, tenure: 15, max_iter: 20000, max_sin_mejora: 3000, prob_intercambio: 0.5}` (struct `ConfigTabu` serde). `validar`: γ ≥ 0 en todas, n_semillas ≥ 1, parámetros tabú > 0, 0 ≤ prob_intercambio ≤ 1. Semillas de corrida: `semilla + k` para k en 0..n_semillas.
5. CLI `reto paso1 [--instancias 1ha,map_27|todas] [--solo-gamma-star]`: escribe en `--resultados`:
   - `paso1_barrido_gamma.csv`: `instancia,map_id,gamma,semilla,D_0..D_9,d_tot,suma_theta_d,suma_c_d,z_paso1,z_opt,z_opt_por_planta,tiempo_paso1_s,iter_paso1,tiempo_opt_s,iter_opt` (una fila por instancia×γ×semilla).
   - `paso1_resumen_gamma.csv`: `instancia,map_id,gamma,D_0..D_9 (de la semilla con menor objetivo del paso 1),suma_theta_d,suma_c_d,z_opt_media,z_opt_min,z_opt_max,z_opt_por_planta_media,tiempo_medio_s,iter_media`.
   - `paso1_composicion.csv` para γ* (mejor semilla): `instancia,map_id,area_ha,d_tot,D_0..D_9,Dnom_0..Dnom_9,L_0..L_9,U_0..U_9`.
   - `paso1_patron_<instancia>.csv` para γ*: `pos,fila,col,x,y,especie` (especie −1 vacía, 0..9).
   - `convergencia_<instancia>.csv` para γ*, mejor semilla: `iteracion,actual,mejor` del paso 1.
   Con `--solo-gamma-star` sólo corre γ* (para los 30 polígonos más adelante). Correr en release.
6. Pruebas: barrido con 2 γ y 2 semillas en 1 ha con parámetros cortos produce 4 filas y D en banda; la regla de extensión (γ extra cuando D* cambia en el extremo); CLI `reto paso1 --instancias 1ha --set …` (parámetros cortos) escribe los CSV con los encabezados exactos.
7. **Correr** `cargo run --release --bin reto -- paso1 --instancias 1ha,map_27` y commitear los CSV de `results/`.

### Task 2: Ajuste de distribuciones (Python)

**Files:** `python/comun.py` (lectura de `data/` y `config/default.json`), `python/ajuste.py`, `python/tests/test_ajuste.py`.

Spec §4 "Ajuste" completo: ρ_i = T_i/A_i; Normal, Gamma (loc=0), Lognormal (loc=0) por MLE con AIC, KS (p) y Shapiro (p); dispersión Poisson con exposición área (χ² Pearson/29) y correlación ρ–área; p̂ = proporción de cada especie en el total; χ² de homogeneidad 10×30 (`scipy.stats.chi2_contingency`); Dirichlet-multinomial: precisión α₀ por MLE (maximizar log-verosimilitud DM sobre α = α₀·p̂, buscar en log α₀); por especie Poisson vs binomial negativa con exposición área (dispersión χ²/gl y AIC; BN con media μ_j·A_i, k por MLE).
Salidas: `results/ajuste_densidad.csv` (modelo, parametros, loglik, AIC, KS_p), `results/ajuste_resumen.csv` (estadístico, valor), `results/ajuste_por_especie.csv`, `results/mc_params.json` (mu = media de ρ, sigma = σ MLE, p_hat), `figures/hist_rho.png` (histograma + 3 densidades).
Valores de referencia (deben salir): media 129.18, sd 3.35, σ MLE 3.29, AIC 160.58/160.40/160.32, Shapiro p 0.21, dispersión 0.41, χ² 286.7 con 261 gl (p 0.13), precisión DM ≈ 15,000, dispersión por especie 0.58–1.45, correlación −0.38. Pruebas pytest de esos valores (redondeo indicado). Si alguno no sale, reportarlo en el informe de la tarea, no forzarlo.

### Task 3: Monte Carlo (Rust) `reto montecarlo`

**Files:** `src/escenarios.rs`, `src/bin/reto.rs`, `Cargo.toml` (`rand_distr = "0.4"`), `tests/fase3.rs`.

`Escenario{omega, rho, t, x:[u32;10], posiciones: Vec<(usize,u8)>}`; `generar(instancia_malla_len, area, map_id, omega, semilla, params_mc) -> Escenario` con el orden de sorteo de Global Constraints. `reto montecarlo [--instancias …|todas]` lee `results/mc_params.json` y `results/paso1_composicion.csv`, genera `n_escenarios` por instancia y escribe:
- `escenarios_X_<instancia>.csv`: `omega,T,X_0..X_9`.
- `mc_resumen_especies.csv`: `instancia,map_id,composicion(nominal|gamma_star),especie,D,E_C,sd_C,ic95_inf,ic95_sup,P90,P95,N_necesario` (N = ceil((1.96·s/0.5)²)).
- `mc_resumen_costo.csv`: `instancia,map_id,composicion,costo_esperado,costo_sd,presupuesto(B·A),P_costo_mayor_B,violaciones_X_mayor_L,escenarios_C_negativo,especies_C_negativo` (costo = Σ c_j·C⁺_j).
Pruebas: misma semilla ⇒ mismo escenario; Σ X = T; posiciones distintas y < |V|; especie de las posiciones coincide con X.
Correr para `1ha,map_27` primero y luego `todas`; commitear CSV.

### Task 4: Paso 2 — compra (Python)

**Files:** `python/paso2_compra.py`, `python/tests/test_paso2.py`.
Lee `escenarios_X_<inst>.csv`, `paso1_composicion.csv`, `data/`. Para cada instancia, ε ∈ {0.10 (D = D* de γ*), 0 (D = D̄ nominal)} y π_factor ∈ {2, 4, 10}: resuelve el PLE (PuLP + HiGHS si `pulp.HiGHS` disponible, si no CBC) y la regla del voceador. Salida `results/paso2_compra.csv`: `instancia,map_id,epsilon,pi_factor,q_0..q_9,costo,costo_ha,presupuesto,cabe,faltante_esperado,sobrante_esperado,objetivo,q_voceador_0..q_voceador_9,costo_voceador`. Pruebas: sin presupuesto el PLE coincide con el voceador (instancia sintética chica); con presupuesto el costo ≤ B·A. Correr para todas las instancias con escenarios.

### Task 5: Paso 3 — ajuste al polígono real (Rust) `reto paso3`

**Files:** `src/busqueda_tabu.rs` (agregar `Restriccion::Cuotas { n: Vec<usize> }`: reasignación a→b con a≠0, b≠0, conteo_b+1 ≤ n_b; intercambios entre libres; el total M no cambia), `src/paso3.rs`, `src/bin/reto.rs`, `tests/fase3.rs`.
Por escenario ω < `n_escenarios_paso3`: regenerar el escenario (misma función de la Task 3); `fija` = posiciones existentes; q de `paso2_compra.csv` (ε = 0.10, π = 2); n_j y M como en Global Constraints; inicial: φ* (`paso1_patron_<inst>.csv`) con existentes sobrescritas; reparar a cuotas (sobrantes → vacía; faltantes ← posiciones vacías libres o especies sobre cuota) hasta Σ = M y conteo_j ≤ n_j; `buscar` con `Cuotas`. Salidas: `paso3_escenarios.csv` (`instancia,map_id,omega,caso,T,V_lib,M,Z,Z_por_planta,tiempo_s,iteraciones`), `paso3_resumen.csv` (`instancia,map_id,n,Z_media,Z_sd,Z_p5,Z_p50,Z_p95,casos_usual,casos_presupuesto_corto,casos_existentes_de_sobra`), y `paso3_patron_<inst>_omega0.csv` (`pos,fila,col,x,y,especie,existente(0/1)`) para los mapas.
Pruebas: **X regenerado en el paso 3 == fila del CSV `escenarios_X`** (ajuste A); las existentes no se mueven; conteos ≤ n_j y Σ = M; delta con `Cuotas` coherente (Z final = recálculo).

### Task 6: Práctica actual — distribución (Rust) `reto practica`

**Files:** `src/practica.rs`, `src/bin/reto.rs`, `tests/fase3.rs`.
Para 1 ha y P1 (y `todas` opcional): variante i (serpentina pura, malla sin existentes, composición de la secuencia), ii (serpentina reparada a la banda), iii (escenario ω = 0 con existentes; la serpentina salta las posiciones ocupadas, composición resultante), tabú sin existentes (patrón del paso 1, γ*) y tabú con existentes (paso 3, ω = 0). Métricas: Z, Z/D_tot, pares inmediatos (anillo 0) de la misma especie, Z por nivel 1–5 y "otro". Salida `results/practica_distribucion.csv` (`instancia,variante,Z,Z_por_planta,pares_inmediatos_misma_especie,Z_nivel1..Z_nivel5,Z_otro`). Prueba: la suma de niveles = Z; variante iii no toca existentes.

### Task 7: Práctica actual — compra, Markov, figuras, RESUMEN y README (Python)

**Files:** `python/practica_compra.py`, `python/markov.py`, `python/figuras.py`, `python/resumen.py`, `python/tests/`, `README.md`.
- Compra (a) nominal 658·A (q = D̄), (b) nominal 524·A (mayor residuo de round(524·A)), (c) ε = 0 paso 2, (d) ε = 0.10 paso 2; en los escenarios: costo/ha, cabe, faltante y sobrante medios, supervivencia Σ s1 p / Σ p y vivas Σ s1 p con p = min(q, C⁺) (C con D̄ en a–c, D* en d). `results/practica_compra.csv`.
- Markov (spec §11) `results/markov.csv`: vivas esperadas a 1 y 2 años por especie para D* y D̄ (1 ha y P1), reposición R_j.
- Figuras (`figures/`, 300 dpi): `hist_rho.png` (si no existe), `C_por_especie_<inst>.png` (boxplots), `curva_compromiso_<inst>.png` (Σθ·D vs Z_opt medio, barras de rango, etiqueta γ), `convergencia_<inst>.png`, `mapa_<inst>.png` (socia vs tabú paso 3 ω=0, existentes con otro marcador), `practica_compra.png`, `practica_distribucion.png`.
- `results/RESUMEN.md`: |V|, round(658·A), excedente y pares por anillo de 1 ha y P1 (y tabla de los 31), tabla de resultados principales de cada paso.
- README con comandos PowerShell exactos del pipeline completo y `run_all.ps1`.

### Task 8 (si hay tiempo): validación CP-SAT, sensibilidades (κ, radios, m_j), DSATUR.
