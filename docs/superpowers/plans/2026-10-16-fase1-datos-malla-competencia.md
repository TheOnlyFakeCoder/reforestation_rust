# Fase 1 — Datos, malla y competencia: plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Datos del reto en CSV, malla tresbolillo en metros con anillos dinámicos, composición
nominal/banda corregida, matriz W de Gower, factor de alcance h y tabla w·h; binario `reto` con
subcomandos `datos` y `malla` que reportan |V| y pares por anillo.

**Architecture:** Módulos nuevos en `src/` al lado de los legacy (`datos`, `config`, `malla`,
`composicion`, `competencia`, `reportes`) y un binario `src/bin/reto.rs`. Todo lee
`data/*.csv` y `config/default.json`; las constantes de `datos.rs` sólo sirven para exportar los CSV.

**Tech Stack:** Rust 2021 (serde, serde_json, csv). Pruebas con `cargo test`.

**Spec:** `docs/superpowers/specs/2026-10-16-entregable2-spec.md` (§ Datos, §1, §2, §3, ajuste B).

## Global Constraints

- d = sqrt(10000 / (0.866 · 658)) ≈ 4.1892 m, configurable.
- Polígono = cuadrado de área A; centros a medio paso del borde; coordenadas en metros.
- D_tot = min(|V|, round(658·A)); D̄ por mayor residuo; L = ceil((1−ε)D̄), U = floor((1+ε)D̄), ε = 0.10.
- w_jj = 1, w_jk = κ·S_jk (κ = 0.6), Gower 7 rasgos con peso 1/7.
- R_j = d (0..6), 2d (7, 9), 2.5d (8), × factor de radios; h = 1 si δ ≤ d.
- Anillos = distancias distintas de la malla < R_max, con tolerancia; vecinos inmediatos siempre incluidos.
- φ ∈ {0..10}: 0 vacía, especie j ↔ j + 1. CSV con idx 0..9.
- No modificar lógica ni pruebas de módulos legacy; sólo comentario `//! LEGACY`.
- Nombres botánicos: *engelmannii*, *streptacantha*.

## Review Focus

1. Puntos exactamente en el borde del cuadrado (x = L o y = L por redondeo) — se incluyen con tolerancia 1e-9 y |V| para 1 ha da 658 exacto (test Task 3).
2. Empates del mayor residuo (especies con δ igual) — desempate por índice menor, determinista (test Task 4, D_tot = 837).
3. ε·D̄ que da entero exacto en punto flotante (D̄ = 10, ε = 0.1) — L = 9, U = 11, sin perder la frontera (test Task 4).
4. Factor de radios que deja R_max ≤ d — los vecinos inmediatos siguen en la vecindad con h = 1 (tests Task 3 y Task 5).
5. CSV con columna faltante o archivo ausente — error claro, no pánico (test Task 1).

---

### Task 1: Datos canónicos y CSV

**Files:**
- Modify: `Cargo.toml` (dependencias serde, serde_json, csv; bin `reto`)
- Create: `src/datos.rs`
- Modify: `src/lib.rs` (registrar módulos)

**Interfaces:**
- Produces:
  - `pub const N_ESPECIES: usize = 10;`
  - `pub struct Especie { nombre: String, delta: u32, c: f64, s1: f64, s2: f64, familia: String, genero: String, forma: String, raiz: u8, cam: bool, fija_n: bool, altura: f64, radio_d: f64 }`
  - `pub struct Poligono { id: String, map_id: u32, area: f64, existentes: [u32; N_ESPECIES] }` con `total_existentes() -> u32`
  - `pub struct Datos { especies: Vec<Especie>, poligonos: Vec<Poligono> }` con `base()`, `densidad_total() -> u32`, `escribir(&Path) -> Result<(), Box<dyn Error>>`, `cargar(&Path) -> Result<Datos, Box<dyn Error>>`, `poligono_por_map_id(u32) -> Option<&Poligono>`
  - Archivos: `especies.csv (idx,especie,delta,c)`, `supervivencia.csv (idx,especie,s0,s1,s2)`, `rasgos.csv (idx,especie,familia,genero,forma,raiz,cam,fija_n,altura_m,radio_d)`, `poligonos.csv (id,map_id,area_ha,X_0..X_9)`

- [ ] **Step 1: Write the failing tests** (al final de `src/datos.rs`, con un `Datos::base()` que aún no existe)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dir_temporal(nombre: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("reto_{}_{}", nombre, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn base_tiene_densidad_658_y_area_183_01() {
        let datos = Datos::base();
        assert_eq!(datos.densidad_total(), 658);
        let area: f64 = datos.poligonos.iter().map(|p| p.area).sum();
        assert!((area - 183.01).abs() < 1e-9);
        assert_eq!(datos.poligonos.len(), 30);
    }

    #[test]
    fn map_id_de_p1_es_27_y_falta_el_29() {
        let datos = Datos::base();
        assert_eq!(datos.poligonos[0].id, "P1");
        assert_eq!(datos.poligonos[0].map_id, 27);
        assert_eq!(datos.poligonos[29].map_id, 1);
        let mut ids: Vec<u32> = datos.poligonos.iter().map(|p| p.map_id).collect();
        ids.sort();
        let esperados: Vec<u32> = (1..=31).filter(|&i| i != 29).collect();
        assert_eq!(ids, esperados);
        assert_eq!(datos.poligono_por_map_id(27).unwrap().id, "P1");
        assert!(datos.poligono_por_map_id(29).is_none());
    }

    #[test]
    fn existentes_de_p1_y_p30_coinciden_con_writer() {
        let datos = Datos::base();
        assert_eq!(datos.poligonos[0].existentes, [8, 46, 16, 16, 11, 14, 18, 12, 15, 9]);
        assert_eq!(datos.poligonos[0].total_existentes(), 165);
        assert_eq!(datos.poligonos[29].existentes, [34, 195, 56, 58, 55, 46, 91, 63, 86, 23]);
    }

    #[test]
    fn nombres_botanicos_corregidos() {
        let datos = Datos::base();
        assert_eq!(datos.especies[5].nombre, "Opuntia engelmannii");
        assert_eq!(datos.especies[7].nombre, "Opuntia streptacantha");
    }

    #[test]
    fn escribir_y_cargar_conserva_los_datos() {
        let dir = dir_temporal("roundtrip");
        let datos = Datos::base();
        datos.escribir(&dir).unwrap();
        let leidos = Datos::cargar(&dir).unwrap();
        assert_eq!(leidos, datos);
    }

    #[test]
    fn cargar_falla_si_falta_un_archivo() {
        let dir = dir_temporal("falta");
        Datos::base().escribir(&dir).unwrap();
        std::fs::remove_file(dir.join("rasgos.csv")).unwrap();
        assert!(Datos::cargar(&dir).is_err());
    }

    #[test]
    fn cargar_falla_si_falta_una_columna() {
        let dir = dir_temporal("columna");
        Datos::base().escribir(&dir).unwrap();
        std::fs::write(dir.join("especies.csv"), "idx,especie,c\n0,A,26\n").unwrap();
        let error = Datos::cargar(&dir).unwrap_err().to_string();
        assert!(error.contains("delta"), "{error}");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --lib datos`
Expected: FAIL de compilación (`Datos` no existe).

- [ ] **Step 3: Implement** `src/datos.rs` (estructuras, constantes de `writer.rs` con nombres corregidos, rasgos de la spec §2, `map_id`, escritura/lectura CSV con búsqueda de columnas por nombre y error `"falta la columna <nombre> en <archivo>"`). Agregar a `Cargo.toml`:

```toml
[[bin]]
name = "reto"
path = "src/bin/reto.rs"

serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"
```

y `pub mod datos;` en `src/lib.rs`. Crear `src/bin/reto.rs` mínimo (`fn main() {}`) para que el bin compile.

- [ ] **Step 4: Run to verify it passes** — `cargo test --lib datos` PASS; `cargo test` completo verde.
- [ ] **Step 5: Commit** — `git commit -m "feat(datos): datos canónicos del reto y CSV"`

### Task 2: Configuración

**Files:** Create `src/config.rs`, `config/default.json`; Modify `src/lib.rs`.

**Interfaces:**
- Produces: `pub fn d_por_densidad(densidad: f64) -> f64`;
  `#[serde(default, deny_unknown_fields)] pub struct Config { d: f64, epsilon: f64, kappa: f64, factor_radios: f64, tolerancia: f64 }`
  con `Default`, `cargar(&Path) -> Result<Config, Box<dyn Error>>`, `aplicar(&mut self, asignacion: &str) -> Result<(), String>` (`"clave=valor"`, valor JSON).

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_por_defecto_es_4_1892_m() {
        assert!((Config::default().d - 4.189171).abs() < 1e-6);
    }

    #[test]
    fn json_parcial_completa_con_defaults() {
        let config: Config = serde_json::from_str(r#"{"kappa": 0.4}"#).unwrap();
        assert_eq!(config.kappa, 0.4);
        assert_eq!(config.epsilon, 0.10);
        assert_eq!(config.factor_radios, 1.0);
    }

    #[test]
    fn clave_desconocida_es_error() {
        assert!(serde_json::from_str::<Config>(r#"{"kapa": 0.4}"#).is_err());
    }

    #[test]
    fn aplicar_sobrescribe_un_campo() {
        let mut config = Config::default();
        config.aplicar("epsilon=0").unwrap();
        assert_eq!(config.epsilon, 0.0);
        config.aplicar("factor_radios=1.25").unwrap();
        assert_eq!(config.factor_radios, 1.25);
    }

    #[test]
    fn aplicar_rechaza_clave_desconocida_o_sin_igual() {
        let mut config = Config::default();
        assert!(config.aplicar("radio=2").is_err());
        assert!(config.aplicar("epsilon").is_err());
        assert_eq!(config, Config::default());
    }
}
```

- [ ] **Step 2:** `cargo test --lib config` → FAIL (no existe).
- [ ] **Step 3: Implement**

```rust
//! Parámetros del modelo, compartidos por Rust y Python vía `config/default.json`.
use std::error::Error;
use std::path::Path;
use serde::{Deserialize, Serialize};

pub fn d_por_densidad(densidad: f64) -> f64 { (10_000.0 / (0.866 * densidad)).sqrt() }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config { pub d: f64, pub epsilon: f64, pub kappa: f64, pub factor_radios: f64, pub tolerancia: f64 }

impl Default for Config { /* d_por_densidad(658), 0.10, 0.6, 1.0, 1e-6 */ }

impl Config {
    pub fn cargar(path: &Path) -> Result<Config, Box<dyn Error>> {
        let texto = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(serde_json::from_str(&texto)?)
    }
    pub fn aplicar(&mut self, asignacion: &str) -> Result<(), String> {
        let (clave, valor) = asignacion.split_once('=').ok_or(format!("se esperaba clave=valor: {asignacion}"))?;
        let mut json = serde_json::to_value(&*self).map_err(|e| e.to_string())?;
        let valor: serde_json::Value = serde_json::from_str(valor.trim()).map_err(|e| e.to_string())?;
        let objeto = json.as_object_mut().unwrap();
        if !objeto.contains_key(clave.trim()) { return Err(format!("clave desconocida: {clave}")); }
        objeto.insert(clave.trim().to_string(), valor);
        *self = serde_json::from_value(json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
```

`config/default.json`: `{"d": 4.189171180993856, "epsilon": 0.10, "kappa": 0.6, "factor_radios": 1.0, "tolerancia": 1e-6}`.

- [ ] **Step 4:** `cargo test` verde. **Step 5:** commit `feat(config): parámetros del modelo en JSON`.

### Task 3: Malla tresbolillo y vecindad por anillos

**Files:** Create `src/malla.rs`; Modify `src/lib.rs`.

**Interfaces:**
- Produces: `pub struct Posicion { fila: usize, col: usize, x: f64, y: f64 }`;
  `pub struct Malla { d: f64, lado: f64, posiciones: Vec<Posicion>, .. }` con
  `cuadrada(area_ha, d)`, `cuadrado(lado_m, d)`, `len()`, `filas()`, `distancia(u, v) -> f64`,
  `vecindad(radio, tol) -> Vecindad`;
  `pub struct Vecino { pos: u32, anillo: u8 }`;
  `pub struct Vecindad { anillos: Vec<f64>, vecinos: Vec<Vec<Vecino>> }` con
  `pares_por_anillo() -> Vec<usize>`, `max_vecinos() -> usize`.
  Regla: v ∈ N(u) si δ ≤ d + tol **o** δ < radio − tol. Anillos = distancias distintas (agrupadas con tol), ordenadas.

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    const D: f64 = 4.189171180993856;

    #[test]
    fn numero_de_posiciones_por_area() {
        for (area, esperado) in [(1.0, 658), (1.28, 837), (8.0, 5265)] {
            assert_eq!(Malla::cuadrada(area, D).len(), esperado, "área {area}");
        }
    }

    #[test]
    fn posiciones_quedan_dentro_del_cuadrado_a_medio_paso_del_borde() {
        let malla = Malla::cuadrada(1.0, D);
        assert!((malla.lado - 100.0).abs() < 1e-9);
        let min_x = malla.posiciones.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = malla.posiciones.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        assert!((min_x - D / 2.0).abs() < 1e-9);
        assert!((min_y - D * 3f64.sqrt() / 4.0).abs() < 1e-9);
        assert!(malla.posiciones.iter().all(|p| p.x <= 100.0 + 1e-9 && p.y <= 100.0 + 1e-9));
    }

    #[test]
    fn malla_chica_tiene_pares_9_4_2() {
        // d = 1, lado 2.2: 3 filas de 2 puntos; contado a mano: 9 pares a d, 4 a √3, 2 a 2.
        let malla = Malla::cuadrado(2.2, 1.0);
        assert_eq!(malla.len(), 6);
        let vecindad = malla.vecindad(2.5, 1e-6);
        assert_eq!(vecindad.pares_por_anillo(), vec![9, 4, 2]);
        assert!((vecindad.anillos[1] - 3f64.sqrt()).abs() < 1e-9);
    }

    fn nodo_interior(malla: &Malla) -> usize {
        let centro = malla.lado / 2.0;
        (0..malla.len())
            .min_by(|&a, &b| {
                let da = (malla.posiciones[a].x - centro).hypot(malla.posiciones[a].y - centro);
                let db = (malla.posiciones[b].x - centro).hypot(malla.posiciones[b].y - centro);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap()
    }

    fn conteo_por_anillo(vecindad: &Vecindad, u: usize) -> Vec<usize> {
        let mut conteo = vec![0; vecindad.anillos.len()];
        for vecino in &vecindad.vecinos[u] { conteo[vecino.anillo as usize] += 1; }
        conteo
    }

    #[test]
    fn nodo_interior_tiene_6_6_6_vecinos_a_d_raiz3d_2d() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(2.5 * D, 1e-6);
        let esperados = [D, 3f64.sqrt() * D, 2.0 * D];
        assert_eq!(vecindad.anillos.len(), 3);
        for (anillo, esperado) in vecindad.anillos.iter().zip(esperados) {
            assert!((anillo - esperado).abs() < 1e-6);
        }
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6, 6, 6]);
        assert_eq!(vecindad.max_vecinos(), 18);
    }

    #[test]
    fn radio_3_125d_agrega_anillos_raiz7_y_3() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(3.125 * D, 1e-6);
        let esperados = [1.0, 3f64.sqrt(), 2.0, 7f64.sqrt(), 3.0];
        assert_eq!(vecindad.anillos.len(), 5);
        for (anillo, esperado) in vecindad.anillos.iter().zip(esperados) {
            assert!((anillo / D - esperado).abs() < 1e-6);
        }
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6, 6, 6, 12, 6]);
    }

    #[test]
    fn radio_menor_a_d_conserva_vecinos_inmediatos() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(0.75 * D, 1e-6);
        assert_eq!(vecindad.anillos.len(), 1);
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6]);
    }

    #[test]
    fn vecindad_es_simetrica() {
        let malla = Malla::cuadrada(1.28, D);
        let vecindad = malla.vecindad(2.5 * D, 1e-6);
        for (u, lista) in vecindad.vecinos.iter().enumerate() {
            for vecino in lista {
                let v = vecino.pos as usize;
                assert!(vecindad.vecinos[v].iter().any(|w| w.pos as usize == u && w.anillo == vecino.anillo));
                assert!((malla.distancia(u, v) - vecindad.anillos[vecino.anillo as usize]).abs() < 1e-6);
            }
        }
    }
}
```

- [ ] **Step 2:** `cargo test --lib malla` → FAIL.
- [ ] **Step 3: Implement**: filas con y = h/2 + f·h (h = d·√3/2) mientras y ≤ lado + 1e-9; x = x0 + c·d con x0 = d/2 en pares y d en impares mientras x ≤ lado + 1e-9; `inicio_fila` para recorrer filas vecinas (|Δf| ≤ floor(radio/h) + 1); recolectar pares (u < v), agrupar distancias ordenadas en anillos (nuevo anillo si δ − inicio > tol), asignar anillo por búsqueda lineal (|δ − a| ≤ tol) y llenar ambas listas.
- [ ] **Step 4:** `cargo test` verde. **Step 5:** commit `feat(malla): tresbolillo en metros con anillos dinámicos`.

### Task 4: Composición nominal y banda

**Files:** Create `src/composicion.rs`; Modify `src/lib.rs`.

**Interfaces:** `pub fn d_tot(n_posiciones: usize, area: f64, densidad: u32) -> usize`;
`pub fn composicion_nominal(d_tot: usize, delta: &[u32]) -> Vec<usize>` (aritmética entera; empates → índice menor);
`pub struct Banda { l: Vec<usize>, u: Vec<usize> }`; `pub fn banda(nominal: &[usize], epsilon: f64) -> Banda`
(L = ceil((1−ε)D̄ − 1e-9), U = floor((1+ε)D̄ + 1e-9)).

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    const DELTA: [u32; 10] = [42, 196, 42, 42, 49, 38, 73, 64, 86, 26];

    #[test]
    fn d_tot_es_el_minimo_entre_posiciones_y_658a() {
        assert_eq!(d_tot(837, 1.28, 658), 837);
        assert_eq!(d_tot(658, 1.0, 658), 658);
        assert_eq!(d_tot(5265, 8.0, 658), 5264);
    }

    #[test]
    fn mayor_residuo_con_658_reproduce_delta() {
        let nominal = composicion_nominal(658, &DELTA);
        assert_eq!(nominal, DELTA.iter().map(|&x| x as usize).collect::<Vec<_>>());
    }

    #[test]
    fn mayor_residuo_para_837_reparte_el_sobrante_con_desempate_por_indice() {
        // pisos 53,249,53,53,62,48,92,81,109,33 (suma 833); sobrante 4 a las fracciones
        // .859 (idx 6) y .4255 (idx 0, 2, 3) — calculado a mano.
        assert_eq!(composicion_nominal(837, &DELTA), vec![54, 249, 54, 54, 62, 48, 93, 81, 109, 33]);
    }

    #[test]
    fn mayor_residuo_con_d_tot_cero() {
        assert_eq!(composicion_nominal(0, &DELTA), vec![0; 10]);
    }

    #[test]
    fn banda_usa_ceil_abajo_y_floor_arriba() {
        let b = banda(&[42, 10, 196, 0], 0.10);
        assert_eq!(b.l, vec![38, 9, 177, 0]);
        assert_eq!(b.u, vec![46, 11, 215, 0]);
    }

    #[test]
    fn banda_con_epsilon_cero_es_la_nominal() {
        let b = banda(&[54, 249, 33], 0.0);
        assert_eq!(b.l, vec![54, 249, 33]);
        assert_eq!(b.u, vec![54, 249, 33]);
    }
}
```

- [ ] **Step 2:** FAIL. **Step 3:** implementar. **Step 4:** verde. **Step 5:** commit `feat(composicion): mayor residuo y banda corregida`.

### Task 5: Competencia — Gower, W, alcance h y tabla w·h

**Files:** Create `src/competencia.rs`; Modify `src/lib.rs`.

**Interfaces:**
- Consumes: `datos::Especie`.
- Produces: `pub const VACIA: u8 = 0;`
  `pub fn similitud_gower(a: &Especie, b: &Especie, ln_rango_altura: f64) -> f64`;
  `pub fn matriz_w(especies: &[Especie], kappa: f64) -> Vec<Vec<f64>>`;
  `pub fn radios(especies: &[Especie], d: f64, factor: f64) -> Vec<f64>`;
  `pub fn radio_max(radios: &[f64], d: f64) -> f64` (= max(max R_j, d));
  `pub fn alcance(delta: f64, d: f64, r_jk: f64, tol: f64) -> f64`;
  `pub struct TablaCompetencia` con `nueva(w, radios, anillos: &[f64], d, tol)`, `peso(anillo: u8, a: u8, b: u8) -> f64` (a, b en codificación φ), `n_anillos()`.

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::datos::Datos;
    const D: f64 = 4.189171180993856;

    fn w() -> Vec<Vec<f64>> { matriz_w(&Datos::base().especies, 0.6) }
    fn r2(x: f64) -> f64 { (x * 100.0).round() / 100.0 }

    #[test]
    fn w_tiene_diagonal_uno_y_es_simetrica() {
        let w = w();
        for j in 0..10 {
            assert_eq!(w[j][j], 1.0);
            for k in 0..10 { assert!((w[j][k] - w[k][j]).abs() < 1e-15); }
        }
    }

    #[test]
    fn w_lechuguilla_salmiana_calculado_a_mano() {
        // 0.6 · (6 + 1 − ln(1.8/0.5)/ln(8/0.5)) / 7 = 0.56040
        assert!((w()[0][1] - 0.56040).abs() < 1e-4);
    }

    #[test]
    fn w_cae_en_los_rangos_de_referencia() {
        let w = w();
        let rango = |pares: Vec<(usize, usize)>, lo: f64, hi: f64| {
            for (j, k) in pares {
                let v = r2(w[j][k]);
                assert!(v >= lo && v <= hi, "w[{j}][{k}] = {} fuera de [{lo}, {hi}]", w[j][k]);
            }
        };
        let pares = |a: std::ops::Range<usize>, b: std::ops::Range<usize>| {
            let mut v = Vec::new();
            for j in a.clone() { for k in b.clone() { if j != k { v.push((j, k)); } } }
            v
        };
        rango(pares(0..4, 0..4), 0.56, 0.59);
        rango(pares(4..8, 4..8), 0.57, 0.59);
        rango(pares(0..4, 4..8), 0.28, 0.34);
        rango(pares(9..10, 0..8), 0.25, 0.34);
        rango(vec![(9, 8)], 0.12, 0.12);
        rango(pares(8..9, 0..8), 0.00, 0.06);
        assert!(w[0][8].abs() < 1e-12);
    }

    #[test]
    fn radios_por_especie_y_factor() {
        let especies = Datos::base().especies;
        let r = radios(&especies, D, 1.0);
        assert!((r[0] - D).abs() < 1e-12 && (r[6] - D).abs() < 1e-12);
        assert!((r[7] - 2.0 * D).abs() < 1e-12 && (r[9] - 2.0 * D).abs() < 1e-12);
        assert!((r[8] - 2.5 * D).abs() < 1e-12);
        assert!((radio_max(&radios(&especies, D, 1.25), D) - 3.125 * D).abs() < 1e-12);
        assert!((radio_max(&radios(&especies, D, 0.75), D) - 1.875 * D).abs() < 1e-12);
        assert!((radio_max(&radios(&especies, D, 0.3), D) - D).abs() < 1e-12);
    }

    #[test]
    fn alcance_pesos_esperados() {
        let s3 = 3f64.sqrt() * D;
        assert!((alcance(s3, D, 2.0 * D, 1e-6) - 0.26795).abs() < 1e-4);
        assert!((alcance(s3, D, 2.5 * D, 1e-6) - 0.51197).abs() < 1e-4);
        assert!((alcance(2.0 * D, D, 2.5 * D, 1e-6) - 0.33333).abs() < 1e-4);
        assert_eq!(alcance(2.0 * D, D, 2.0 * D, 1e-6), 0.0);
        assert_eq!(alcance(D * (1.0 + 1e-9), D, D, 1e-6), 1.0);
    }

    #[test]
    fn alcance_inmediato_es_uno_aun_con_radio_menor_a_d() {
        assert_eq!(alcance(D, D, 0.75 * D, 1e-6), 1.0);
        assert_eq!(alcance(3f64.sqrt() * D, D, 0.75 * D, 1e-6), 0.0);
    }

    #[test]
    fn tabla_peso_es_w_por_h_y_cero_con_vacia() {
        let especies = Datos::base().especies;
        let w = w();
        let anillos = [D, 3f64.sqrt() * D, 2.0 * D];
        let tabla = TablaCompetencia::nueva(&w, &radios(&especies, D, 1.0), &anillos, D, 1e-6);
        assert_eq!(tabla.n_anillos(), 3);
        assert_eq!(tabla.peso(0, 1, 1), 1.0);
        assert!((tabla.peso(1, 8, 10) - w[7][9] * 0.26795).abs() < 1e-4);
        assert!((tabla.peso(2, 9, 9) - 0.33333).abs() < 1e-4);
        assert_eq!(tabla.peso(2, 8, 10), 0.0);
        for anillo in 0..3 { for a in 0..=10 {
            assert_eq!(tabla.peso(anillo, VACIA, a), 0.0);
            assert_eq!(tabla.peso(anillo, a, VACIA), 0.0);
        } }
    }
}
```

- [ ] **Step 2:** FAIL. **Step 3:** implementar (tabla plana `[anillo][a][b]` de 11×11 por anillo). **Step 4:** verde. **Step 5:** commit `feat(competencia): W de Gower, alcance h y tabla w·h`.

### Task 6: Reportes, binario `reto` (datos, malla), comentarios LEGACY

**Files:** Create `src/reportes.rs`; Modify `src/bin/reto.rs`, `src/lib.rs`; comentarios `//! LEGACY` en
`src/tabu.rs`, `src/interaction.rs`, `src/competition.rs`, `src/map.rs`, `src/algorithm.rs`, `src/bin/writer.rs`.

**Interfaces:**
- Produces: `pub struct FilaMalla { instancia: String, map_id: Option<u32>, area: f64, n_pos: usize, objetivo: usize, diferencia: i64, d_tot: usize, anillos_m: Vec<f64>, pares: Vec<usize> }`;
  `pub fn resumen_malla(datos: &Datos, config: &Config) -> Vec<FilaMalla>` (primero "1ha", luego los 30 por orden);
  `pub fn escribir_resumen_malla(path, &[FilaMalla])`; `pub fn escribir_anillos(path, &[FilaMalla])`;
  `pub fn escribir_w(path, especies, w)`.
- CLI: `reto <datos|malla> [--datos DIR] [--resultados DIR] [--config PATH] [--set clave=valor]...`

- [ ] **Step 1: Failing tests** (`src/reportes.rs`)

```rust
#[test]
fn resumen_malla_1ha_y_p1() {
    let filas = resumen_malla(&Datos::base(), &Config::default());
    assert_eq!(filas.len(), 31);
    assert_eq!((filas[0].instancia.as_str(), filas[0].n_pos, filas[0].objetivo, filas[0].diferencia), ("1ha", 658, 658, 0));
    let p1 = filas.iter().find(|f| f.map_id == Some(27)).unwrap();
    assert_eq!((p1.n_pos, p1.objetivo, p1.diferencia, p1.d_tot), (837, 842, -5, 837));
    assert_eq!(p1.pares.len(), 3);
}
```

- [ ] **Step 2:** FAIL. **Step 3:** implementar reportes y CLI (`datos` escribe `Datos::base()` en `--datos`; `malla` carga datos/config, escribe `malla_resumen.csv`, `malla_anillos.csv`, `W.csv` y muestra tabla de 1 ha y P1). Comentarios LEGACY.
- [ ] **Step 4:** verde. **Step 5:** commit `feat(reto): subcomandos datos y malla`.

### Task 7: Test de fase

**Files:** Create `tests/fase1.rs`.

- [ ] **Step 1: Tests de fase**

```rust
// 1. CLI de punta a punta en directorios temporales: `reto datos` y `reto malla`;
//    W.csv 10 filas × 11 columnas con W[lechuguilla][Prosopis] = 0; malla_resumen.csv 31 filas;
//    P1 (map_id 27) con 837 posiciones.
// 2. Para los 30 polígonos: |V|/(658A) ∈ [0.985, 1.015], Σ D̄ = D_tot, L ≤ D̄ ≤ U, max vecinos 18.
// 3. Factor 1.25: Prosopis-Prosopis con peso > 0 en √7·d (≈0.2255) y 3d (≈0.0588).
// 4. Factor 0.75: R_max = 1.875d, 2 anillos, peso inmediato = w para todo par, Prosopis a √3·d ≈ 0.1634.
// 5. `reto` con subcomando desconocido o clave --set inválida termina con código ≠ 0.
```

- [ ] **Step 2:** `cargo test` completo verde, `cargo clippy --all-targets` sin warnings nuevos.
- [ ] **Step 3:** correr `cargo run --release --bin reto -- datos` y `-- malla`; commitear `data/`, `results/` de fase 1.
