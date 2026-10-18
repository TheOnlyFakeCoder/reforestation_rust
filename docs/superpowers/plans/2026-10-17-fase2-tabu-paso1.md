# Fase 2 — Búsqueda tabú y paso 1 (barrido de γ): plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. tdd-guard estricto: una prueba en rojo por ciclo, stubs primero, pruebas nuevas al final del módulo.

**Goal:** Motor tabú nuevo (reasignación + intercambio, banda, tenure, candidatos, aspiración, paro
por iteraciones sin mejora, posiciones fijas) con delta incremental verificado; paso 1 con barrido
de γ en 1 ha y P1, reoptimización de Z con composición fija (ajuste C), CSV y figura de la curva de
compromiso. **Se detiene al mostrar la curva** (no se corren los 30 polígonos).

**Architecture:** `objetivo.rs` (Z y deltas sobre una vecindad CSR), `busqueda_tabu.rs` (motor
genérico con `Restriccion::{Banda, ComposicionFija}`; el modo `Cuotas` del paso 3 llega en su fase),
`paso1.rs` (θ, serpentina, reparación a banda, resolver y barrido), `reto paso1` en el CLI,
`python/figuras.py` para la curva. `tabu.rs` legacy no se toca.

**Tech Stack:** Rust (+ `rand 0.8`, `rand_chacha 0.3`); Python (.venv: pandas, matplotlib).

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` §5, §8, ajuste C; instrucciones
del usuario 2026-10-17 punto 5.

## Global Constraints

- Paso 1: min γ·Z(φ) + Σ θ_j·D_j; |φ⁻¹(j)| = D_j, Σ D_j = D_tot, L_j ≤ D_j ≤ U_j; malla sin existentes; c_j no entra.
- θ_j = m_j·(1 − s_j), m_j = c_j·m_escala (1 por defecto; 0, 0.5, 2 sensibilidad), s_j = s1_j (s2 configurable).
- Vacía (φ = 0): cantidad fija |V| − D_tot; sólo se mueve con intercambios.
- Reasignación a→b en u libre: permitida si D_a − 1 ≥ L_a y D_b + 1 ≤ U_b; delta = γ·ΔZ + θ_b − θ_a.
- Intercambio u↔v libres con φu ≠ φv: delta = γ·ΔZ (fórmula de la spec, excluye v de N(u) y u de N(v)).
- Tabú: al dejar u la especie s, prohibido volver u→s durante t iteraciones; aspiración si mejora la mejor; se aplica el mejor no tabú aunque empeore; paro por max_iter o max_sin_mejora.
- Posiciones fijas (existentes) nunca cambian; sí cuentan como vecinas.
- Prueba del delta: 10,000 movimientos aleatorios de ambos tipos, con vacías y fijas, delta vs recálculo completo, tolerancia 1e-9.
- Barrido γ ∈ {0, 0.1, 0.3, 1, 3, 10, 30, 100}; si D* cambia entre los dos γ mayores, agregar γ·3 (máx. 3 veces). ≥ 3 semillas por γ.
- Por γ: D*(γ), Σθ·D, Σc·D, Z del paso 1, Z_opt(D*) (ajuste C, sólo intercambios), Z_opt/D_tot, tiempo e iteraciones de ambas búsquedas; media y rango de Z_opt entre semillas.
- `cargo test` normal debe pasar. Commits sin línea de atribución.

## Review Focus

1. Intercambio entre u y v que son vecinos entre sí — el par (u, v) no cambia de peso y no debe contarse dos veces (prueba del delta con pares adyacentes forzados).
2. Reasignación cuando una especie está justo en L o en U — el candidato se descarta, no se aplica (prueba de banda en cada iteración).
3. Instancia sin posiciones vacías (excedente 0) — el motor no se cuelga buscando intercambios con vacías.
4. γ = 0 — el paso 1 debe llegar al óptimo lineal de Σθ·D (1 ha: D* = [38, 215, 38, 46, 46, 35, 80, 58, 78, 24]).
5. Todas las posiciones de una especie fijas — la reasignación no puede elegir una fija.

---

### Task 1: Objetivo Z y deltas incrementales

**Files:** Create `src/objetivo.rs`; Modify `Cargo.toml` (rand, rand_chacha), `src/lib.rs`.

**Interfaces:** Produces
`pub struct Grafo { inicio: Vec<u32>, vecinos: Vec<Vecino> }` con `Grafo::desde(&Vecindad)`, `vecinos(u) -> &[Vecino]`;
`pub fn z_total(phi: &[u8], grafo: &Grafo, tabla: &TablaCompetencia) -> f64`;
`pub fn delta_reasignacion(phi, u, b, grafo, tabla) -> f64`;
`pub fn delta_intercambio(phi, u, v, grafo, tabla) -> f64`.

Tests: Z de una malla chica calculada a mano (malla 3 filas × 2, d = 1, dos especies);
`delta_coincide_con_recalculo_en_10000_movimientos` (1 ha, 40 vacías, 100 fijas, semilla fija,
mitad reasignaciones y mitad intercambios, 10 % de intercambios forzados entre vecinos; |delta − (Z₁ − Z₀)| < 1e-9).

### Task 2: Motor tabú

**Files:** Create `src/busqueda_tabu.rs`; Modify `src/lib.rs`.

**Interfaces:** Produces
`pub enum Restriccion { Banda { l: Vec<usize>, u: Vec<usize> }, ComposicionFija }`;
`pub struct ParamsTabu { candidatos, tenure, max_iter, max_sin_mejora, prob_intercambio: f64, semilla: u64 }`;
`pub struct Problema<'a> { grafo: &'a Grafo, tabla: &'a TablaCompetencia, fija: Vec<bool>, gamma: f64, theta: [f64; N_ESPECIES], restriccion: Restriccion }`;
`pub struct Resultado { phi, conteos: [usize; 11], z, objetivo, iteraciones, mejoras, tiempo_s, curva: Vec<(usize, f64)> }`;
`pub fn buscar(problema: &Problema, inicial: Vec<u8>, params: &ParamsTabu) -> Resultado`.

Tests (uno por ciclo): mejora o iguala el objetivo inicial; `z` y `objetivo` del resultado coinciden con
el recálculo; la banda se respeta en todas las iteraciones (debug_assert en el motor + conteos finales);
`ComposicionFija` conserva los conteos; las fijas no cambian; misma semilla ⇒ mismo resultado;
instancia sin vacías termina; γ = 0 en 1 ha llega a D* = [38, 215, 38, 46, 46, 35, 80, 58, 78, 24].

### Task 3: Paso 1

**Files:** Create `src/paso1.rs`; Modify `src/lib.rs`.

**Interfaces:** `pub fn theta(especies, m_escala: f64, usar_s2: bool) -> [f64; 10]`;
`pub fn patron_serpentina(malla: &Malla, secuencia: &[usize], d_tot: usize) -> Vec<u8>`;
`pub fn reparar_a_banda(phi: &mut [u8], banda: &Banda)`;
`pub struct FilaPaso1 { instancia, gamma, semilla, d: [usize; 10], suma_theta_d, suma_c_d, z_paso1, z_opt, z_opt_por_planta, tiempo_paso1_s, iter_paso1, tiempo_opt_s, iter_opt }`;
`pub fn resolver_paso1(...) -> (FilaPaso1, Vec<u8>)`; `pub fn barrido_gamma(...) -> Vec<FilaPaso1>`.

Tests: θ calculado a mano (θ₀ = 26·(1 − 0.9435) = 1.469, θ₈ = 26.5·(1 − 0.8214) = 4.7329);
serpentina invierte el sentido en filas impares y deja D_tot plantas; reparar deja L ≤ D ≤ U y Σ = D_tot;
ajuste C: Z_opt ≤ Z del paso 1 y D no cambia; barrido extiende γ cuando D* cambia en el extremo.

### Task 4: Configuración y CLI `reto paso1`

**Files:** Modify `src/config.rs`, `config/default.json`, `src/bin/reto.rs`, `tests/cli.rs`.

Config: `gammas`, `n_semillas` (3), `semilla` (20261017), `m_escala` (1), `usar_s2` (false),
`tabu: { candidatos, tenure, max_iter, max_sin_mejora, prob_intercambio }`; `validar` agrega γ ≥ 0,
n_semillas ≥ 1 y parámetros tabú > 0. CLI `reto paso1 --instancias 1ha,map_27` escribe
`results/paso1_barrido_gamma.csv` (una fila por instancia, γ y semilla) y
`results/paso1_resumen_gamma.csv` (media, mínimo y máximo de Z_opt por instancia y γ).

### Task 5: Figura y corrida

**Files:** Create `python/figuras.py`, `python/tests/test_figuras.py`.

`python python/figuras.py curva` lee `paso1_resumen_gamma.csv` y genera
`figures/curva_compromiso_<instancia>.png` (Σθ·D contra Z_opt medio, barras de rango, etiquetas γ;
300 dpi). Correr el barrido en 1 ha y P1, mostrar la curva al usuario y **DETENERSE**.
