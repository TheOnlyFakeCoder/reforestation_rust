# Fase 1b — Malla rectangular, validación de configuración y ajustes: plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. tdd-guard estricto: una prueba en rojo por ciclo, stubs primero.

**Goal:** D_tot = round(658·A) en todos los polígonos con una malla rectangular casi cuadrada de
excedente mínimo; `Config::validar()`; anillos en `u16`; `debug_assert` de Σδ = 658; tabla de
pesos h exportada para confirmar el ajuste B.

**Architecture:** `malla.rs` gana `Disposicion` (Completa/Alterna), `Malla::rectangular` y
`Malla::para_objetivo`; `reportes.rs` usa `para_objetivo` y agrega columnas; `config.rs` gana
`validar()`, llamado por `cargar` y `aplicar`; el CLI hereda los errores.

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` + instrucciones del usuario del
2026-10-17 (puntos 1–4).

## Global Constraints

- Con d fijo: |V| ≥ round(658·A), excedente mínimo, ancho/alto ∈ [0.8, 1.25], desempate la proporción más cercana a 1.
- Centros a d/2 del borde (como mínimo): filas a h/2 del borde inferior y superior.
- D_tot = round(658·A); excedente |V| − D_tot = posiciones vacías.
- Disposiciones (ruling, ver ledger): **Completa** — c puntos en todas las filas, ancho (c + ½)·d;
  **Alterna** — c en filas pares y c − 1 en impares, ancho c·d. Alto = r·h, h = d·√3/2.
- Tope de excedente en la prueba de fase: 8 (máximo observado: map 7, 4140 contra 4132).
- `cargo test` sin reporter ni RUSTC_BOOTSTRAP debe pasar.

## Review Focus

1. Polígono cuyo mejor rectángulo es de disposición Completa con r impar — la última fila completa sigue a h/2 del borde superior.
2. Empate exacto de excedente y proporción entre Completa y Alterna — desempate determinista (menos columnas primero).
3. `--set d=0` o `tolerancia=-1` en el CLI — error claro, sin colgarse.
4. Más de 255 anillos (radio grande) — índice de anillo correcto (u16).
5. `composicion_nominal` con Σδ ≠ 658 en debug — falla el `debug_assert`.

---

### Task 1: Malla rectangular y búsqueda de dimensiones

**Files:** Modify `src/malla.rs`.

**Interfaces:** Produces
`pub enum Disposicion { Completa, Alterna }`;
`Malla { d, ancho, alto, columnas, disposicion, posiciones, .. }` (reemplaza `lado` por `ancho`/`alto`);
`Malla::rectangular(columnas: usize, filas: usize, disposicion: Disposicion, d: f64) -> Malla`;
`Malla::para_objetivo(objetivo: usize, d: f64) -> Malla`. `cuadrada`/`cuadrado` se conservan.

Tests (uno por ciclo):
```rust
#[test]
fn rectangular_alterna_tiene_c_y_c_menos_1_por_fila() {
    let m = Malla::rectangular(3, 3, Disposicion::Alterna, 1.0);
    assert_eq!(m.len(), 3 + 2 + 3);
    assert!((m.ancho - 3.0).abs() < 1e-12 && (m.alto - 3.0 * SENO_60).abs() < 1e-12);
}
#[test]
fn rectangular_completa_tiene_c_por_fila_y_ancho_c_mas_medio() {
    let m = Malla::rectangular(3, 3, Disposicion::Completa, 1.0);
    assert_eq!(m.len(), 9);
    assert!((m.ancho - 3.5).abs() < 1e-12);
}
#[test]
fn centros_quedan_al_menos_a_medio_paso_del_borde() { /* ambas disposiciones, x ∈ [d/2, ancho − d/2], y ∈ [h/2, alto − h/2] */ }
#[test]
fn para_objetivo_da_excedente_minimo_con_proporcion_valida() {
    // Calculado con búsqueda independiente (Python): (objetivo, |V|, columnas, filas, disposición)
    for (objetivo, v, c, r, disp) in [(658, 658, 24, 28, Alterna), (842, 842, 26, 33, Alterna),
                                       (967, 969, 29, 34, Alterna), (4132, 4140, 60, 69, Completa)] { .. }
}
```

### Task 2: Reportes con la malla rectangular y tabla de pesos h

**Files:** Modify `src/reportes.rs`, `src/bin/reto.rs`, `tests/cli.rs`.

**Interfaces:** `FilaMalla` agrega `columnas, filas, disposicion, ancho_m, alto_m, excedente` (quita
`diferencia`); `d_tot = objetivo`. CSV `malla_resumen.csv`:
`instancia,map_id,area_ha,columnas,filas,disposicion,ancho_m,alto_m,n_pos,round_658A,excedente,d_tot,pares_anillo_1..k`.
`pub fn escribir_alcance(path, especies, config, anillos)` → `results/alcance_h.csv`
(`R_j_d,R_k_d,R_jk_d,anillo,delta_sobre_d,h`) para cada par de clases de radio.

### Task 3: Config::validar y longitudes consistentes

**Files:** Modify `src/config.rs`, `src/competencia.rs`.

`Config::validar(&self) -> Result<(), String>` (d > 0, tolerancia > 0, 0 ≤ ε < 1, κ ≥ 0,
factor_radios > 0); `cargar` y `aplicar` la llaman. `TablaCompetencia::nueva` verifica
`radios.len() == w.len()` y filas cuadradas (assert con mensaje). γ ≥ 0 se agrega con el campo en la fase 2.
CLI: `reto malla --set d=0` termina con error que menciona `d`.

### Task 4: Anillos en u16 y debug_assert de Σδ

**Files:** Modify `src/malla.rs`, `src/competencia.rs`, `src/composicion.rs`.

Tests: vecindad con radio 40 en malla alterna 50×50 con d = 1 tiene > 255 anillos y cada vecino
apunta al anillo de su distancia; `#[cfg(debug_assertions)] #[should_panic]`
`composicion_nominal(10, &[1, 2])`.

### Task 5: Prueba de fase y regeneración de resultados

**Files:** Modify `tests/fase1.rs`; regenerar `results/`.

`0 ≤ |V| − round(658·A) ≤ 8` y D_tot = round(658·A) para 1 ha y los 30 polígonos; proporción en
[0.8, 1.25]. `cargo test` normal verde; `reto malla`.
