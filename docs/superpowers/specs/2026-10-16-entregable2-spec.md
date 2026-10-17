# Especificación — Reto CONAFOR, Entregable 2 (MA2004B Optimización Estocástica)

Fuente: instrucciones del usuario (2026-10-16) + ajustes aprobados. Este documento es la referencia
para todos los planes de fase en `docs/superpowers/plans/`.

## Reglas de trabajo

- No borrar archivos ni funciones existentes; lo reemplazado queda marcado `LEGACY` o se crea al lado.
- Rust: malla, matriz de competencia, búsqueda tabú, escenarios Monte Carlo, evaluación de práctica actual.
- Python (numpy, scipy, pandas, matplotlib, PuLP+HiGHS/CBC, OR-Tools CP-SAT): ajuste de
  distribuciones, modelo de compra, validación exacta, figuras.
- Intercambio por CSV/JSON en `data/` y `results/`. Semillas fijas y configurables.
- Pruebas unitarias para todo lo nuevo (TDD), en especial el delta incremental de la búsqueda tabú.
- README con comandos para Windows (PowerShell).
- Trabajo por fases; cada fase dividida en tareas con tests propios y un test de fase al final.

## Datos

Especies (idx 0..9): δ_j (plantas/ha), c_j ($), s1_j, s2_j

| idx | especie | δ | c | s1 | s2 |
|---|---|---|---|---|---|
| 0 | Agave lechuguilla | 42 | 26.0 | 0.9435 | 0.9265 |
| 1 | Agave salmiana | 196 | 26.0 | 0.9587 | 0.9082 |
| 2 | Agave scabra | 42 | 26.0 | 0.8245 | 0.7453 |
| 3 | Agave striata | 42 | 26.0 | 0.9517 | 0.9138 |
| 4 | Opuntia cantabrigiensis | 49 | 18.0 | 0.9272 | 0.8287 |
| 5 | Opuntia engelmannii | 38 | 21.0 | 0.8986 | 0.8194 |
| 6 | Opuntia robusta | 73 | 18.0 | 0.9576 | 0.9198 |
| 7 | Opuntia streptacantha | 64 | 18.0 | 0.8860 | 0.8525 |
| 8 | Prosopis laevigata | 86 | 26.5 | 0.8214 | 0.8179 |
| 9 | Yucca filifera | 26 | 26.0 | 0.8843 | 0.7263 |

Σδ = 658. 30 polígonos (`plant_matrix`, `areas` de `src/bin/writer.rs`), Σ área = 183.01 ha.
`map_id` de P1..P30: 27, 28, 30, 20, 21, 22, 23, 24, 18, 17, 16, 19, 25, 26, 13, 31, 14, 15, 7, 8, 9,
10, 11, 12, 6, 2, 3, 4, 5, 1 (el 29 del mapa, 6.54 ha, no tiene datos). Reportes usan `map_id`.
Presupuesto B = 12,500 $/ha (B·A con el área real). Sin reposición.
Archivos: `data/especies.csv`, `data/poligonos.csv`, `data/supervivencia.csv` (+ `data/rasgos.csv`).

Notación: J, V, d, D_j, q_j, X_j, C_j = D_j − X_j, w_jk, h_jk(δ), θ_j, γ, φ(u) (0 = vacía).

## 1. Malla

Tresbolillo, d = sqrt(10000/(0.866·658)) ≈ 4.19 m (configurable). Polígono = cuadrado de área A.
Coordenadas reales en metros. |V| ≈ 658·A (reportar |V| y pares por anillo para 1 ha y P1; en
RESUMEN.md por polígono |V|, round(658·A) y diferencia). D_tot = min(|V|, round(658·A)).
D̄_j por mayor residuo (Σ D̄_j = D_tot). L_j = ceil((1−ε)D̄_j), U_j = floor((1+ε)D̄_j), ε = 0.10
(configurable; ε = 0 referencia).

## 2. Matriz W

w_jj = 1; w_jk = κ·S_jk (κ = 0.6; sensibilidad 0.4, 0.8). Gower con 7 rasgos (peso 1/7):
categóricos familia, género, forma, CAM, fija N; raíz (1/2/3): 1 − |r_j − r_k|/2;
altura: 1 − |ln h_j − ln h_k|/ln(h_max/h_min).

| especies | familia | género | forma | raíz | CAM | fija N | h (m) |
|---|---|---|---|---|---|---|---|
| 0..3 | Asparagaceae | Agave | roseta | 1 | sí | no | 0.5, 1.8, 1.2, 0.8 |
| 4..7 | Cactaceae | Opuntia | cladodios | 1 | sí | no | 1.5, 1.2, 2.5, 3.5 |
| 8 | Fabaceae | Prosopis | árbol | 3 | no | sí | 8.0 |
| 9 | Asparagaceae | Yucca | arborescente | 2 | sí | no | 7.0 |

Referencias (2 decimales): Agave-Agave 0.56–0.59; Opuntia-Opuntia 0.57–0.59; Agave-Opuntia
0.28–0.34; Yucca-suculentas 0.25–0.34; Yucca-Prosopis 0.12; Prosopis-suculentas 0.00–0.06;
lechuguilla-Prosopis 0.00 exacto. Exportar `results/W.csv`. La matriz legacy
(`combine_matrices(...)`) queda como legacy.

## 3. Alcance

R_j = d (0..6), 2d (7, 9), 2.5d (8). R_jk = max(R_j, R_k).
h(δ) = 1 si δ ≤ d; 1 − (δ−d)/(R_jk−d) si d < δ < R_jk; 0 si δ ≥ R_jk (tolerancia numérica).
Competencia de par: w_jk·h_jk(δ). Pesos esperados: 0.268 (2.º anillo, streptacantha/Yucca);
0.511 y 0.333 (Prosopis a √3·d y 2d). Sensibilidad: R_j × 0.75 y × 1.25; vecinos inmediatos
siempre h = 1. Z = Σ_{pares {u,v}, δ<R_max} w·h, cada par una vez, vacía con w = 0.

**Ajuste B (anillos dinámicos):** los anillos se generan de todas las distancias distintas de la
malla menores que R_max (con factor de radios), con tolerancia. Pruebas: factor 1.25 → Prosopis con
peso > 0 en √7·d y 3d; factor 0.75 → h = 1 en inmediatos y R_max correcto.

## 4. Distribuciones y Monte Carlo

Ajuste (Python) de ρ_i = T_i/A_i: Normal, Gamma, Lognormal (MLE; AIC, KS, Shapiro); dispersión
Poisson con exposición área; correlación densidad-área; χ² de homogeneidad 10×30;
Dirichlet-multinomial (precisión); por especie Poisson vs binomial negativa.
Referencias: media ρ 129.18; s 3.35 (σ MLE 3.29); AIC 160.58/160.40/160.32; Shapiro p 0.21;
dispersión 0.41; χ² 286.7 con 261 gl (p 0.13); precisión DM ≈ 15,000; dispersión por especie
0.58–1.45; correlación −0.38.

Escenario ω: ρ ~ N(129.18, 3.29²), T = round(ρ·A); X ~ Mult(T, p̂); T posiciones al azar sin
reemplazo (existentes fijas); C_j = D_j − X_j; verificar X_j ≤ L_j (violaciones esperadas 0).
Salidas por especie: E[C_j], sd, IC95, P90, P95, costo esperado, P(costo > B·A), N ≥ (z·s/E)² con
E = 0.5. |Ω| = 1,000.

**Ajuste A (fuente única):** Rust genera los escenarios (`reto montecarlo`) y exporta
`results/escenarios_X_<map_id>.csv` (omega, T, X_0..X_9). Python solo lee. Posiciones regeneradas
deterministamente con (semilla base, polígono, ω). Paso 3 usa los primeros N (≥100) del mismo
conjunto. Prueba: X regenerado en paso 3 ≡ CSV.

## 5. Paso 1 (Rust, tabú) — composición

min γ·Z(φ) + Σ θ_j D_j s.a. |φ⁻¹(j)| = D_j, Σ D_j = D_tot, L_j ≤ D_j ≤ U_j (malla sin existentes).
θ_j = m_j(1 − s_j), m_j = c_j, s_j = s1_j (configurable s2; m × 0, 0.5, 2). c_j no entra.
Barrido γ ∈ {0, 0.1, 0.25, 0.5, 1, 2, 5}: D*, Σθ·D, Z, Z/D_tot, Σc·D.

**Ajuste C:** para cada γ, segunda búsqueda (solo intercambios) minimizando Z con D*(γ) fija →
Z_opt(D*), Z_opt/D_tot; la curva de compromiso usa Z_opt.
γ* = 1 provisional en config; mostrar curva (1 ha, P1) al usuario antes de correr los 30 polígonos.
Paso 1 por polígono.

## 6. Paso 2 (Python, PLE con escenarios) — compra

min Σ c q + (1/|Ω|) Σ_ω Σ_j (π f + η e); f ≥ C − q, e ≥ q − C, f, e ≥ 0; Σ c q ≤ B·A; q entero.
π = 2c, η = 0 (sens. 4c, 10c). Comparar con voceador sin presupuesto (cuantil (π−c)/(π+η)).
1 ha y P1, ε = 0.10 y ε = 0.

## 7. Paso 3 (Rust, tabú) — ajuste al polígono real

n_j = min(q_j, C_jω) (C truncado a 0); M = min(|V_lib|, Σ n_j). min Z (nueva-nueva y
nueva-existente). Casos: usual, presupuesto corto (huecos), existentes de sobra (contar escenarios y
especies; se esperan 0). Inicial: φ* con existentes sobrescritas + reparación. ≥ 100 escenarios por
polígono: media, sd, percentiles de Z.

## 8. Búsqueda tabú

Reasignación (paso 1: D_a ≥ L_a y D_b ≤ U_b tras el cambio; paso 3: b con cuota; vacía fija, solo
intercambios) e intercambio (nuevo). Delta incremental sobre vecinos con pesos h (vecinos incluyen
existentes); paso 1 reasignación: γΔZ + θ_b − θ_a. Prueba obligatoria: miles de movimientos
aleatorios, delta = recálculo. K candidatos aleatorios; tabú posición→especie anterior durante t;
aspiración; mejor no tabú aunque empeore. Calibrar K, t, I, máx. iter con curvas de convergencia
(P1 y 8 ha) en `figures/`. Varias semillas: mejor, promedio, sd. Guardar tiempo e iteraciones de
cada corrida.

## 9. Validación exacta (Python, CP-SAT)

~10 instancias de 50–100 posiciones con existentes, ≥ 1 con huecos. Formulación del paso 3.
Reportar Z*, cota, tiempo, Z_TS, brecha. **Ajuste D:** coeficientes w·h × 10,000 redondeados en el
modelo; Z*, Z_TS y brecha con valores originales; reportar error máximo de redondeo.

## 10. Práctica actual

Compra: (a) nominal 658·A; (b) nominal 524·A; (c) ε = 0 paso 2; (d) ε = 0.10 paso 2 — costo/ha,
cabe en presupuesto, faltante y sobrante promedio, supervivencia esperada Σ s_j p_j / Σ p_j con
p_j = min(q_j, C_j), y plantas vivas esperadas al año Σ s_j p_j.
Distribución: secuencia de 21 (`DEFAULT_ROW_SEQUENCE`) en serpentina — i sin existentes (composición
de la secuencia), ii sin existentes corregida a la banda, iii con existentes (salta ocupadas y
continúa; composición resultante). Contra DSATUR por cuotas y tabú: Z, Z/D_tot, pares inmediatos
misma especie, Z por nivel (1 misma especie, 2 mismo género, 3 w ∈ [0.2, 0.5], 4 Yucca-Prosopis,
5 Prosopis-suculentas). Normalización por ha y Z/D_tot.

## 11. Markov (extensión)

Viva/muerta; muerte año 1 = 1 − s1, año 2 = 1 − s2/s1. Proyección a 1 y 2 años (elegida y
nominal). R_j = (1 − s1) + (s1 − s2) + (1 − s1)². No entra en optimización.

## 12. Figuras (PNG 300 dpi, `figures/`)

Histograma ρ + densidades; distribución C_j; curva compromiso vs γ; convergencia tabú; mapas
(socia vs tabú, existentes con otro símbolo); barras práctica actual. Opcional: UI antes/después.

## 13. Problemas a corregir

Matriz ≈ identidad; radio sin fuente/decaimiento; malla 1.7 m/1.6; 658/ha ignorando existentes;
tabú sin intercambio y corte a 60; banda floor/ceil invertidos; Markov estacionario no reportar.

## Decisiones aprobadas

- `.venv` con uv + `requirements.txt`; `.venv` en `.gitignore`.
- `tabu.rs` legacy intacto (lógica y prueba); sólo comentario LEGACY.
- Alcance: análisis completo para 1 ha y P1 (map_id 27); 30 polígonos con parámetros por defecto
  (≥ 100 escenarios) para la tabla por hectárea.
- P1 con 837 posiciones (−0.6 %) aceptado.
- Codificación interna φ ∈ {0..10}: 0 vacía, especie j ↔ j+1; CSV con idx 0..9.
