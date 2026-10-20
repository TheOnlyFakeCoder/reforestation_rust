//! Instancias pequeñas de validación: se exportan a JSON para que Python (CP-SAT) calcule el
//! óptimo exacto y se comparan contra la búsqueda tabú.
//!
//! En JSON y CSV las especies van de 0 a 9 y −1 es vacía/libre; internamente φ usa la
//! codificación de `competencia` (0 = vacía, especie j ↔ j + 1).

use std::error::Error;
use std::path::Path;

use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::busqueda_tabu::{buscar, MotivoParo, ParamsTabu, Problema, Restriccion};
use crate::competencia::{matriz_w, radio_max, radios, TablaCompetencia};
use crate::composicion::{banda, composicion_nominal, Banda};
use crate::config::{Config, ConfigTabu};
use crate::datos::{Datos, N_ESPECIES};
use crate::escenarios::ParamsMc;
use crate::malla::{Disposicion, Malla, Vecindad};
use crate::objetivo::Grafo;
use crate::paso1::{resolver_paso1, theta, Instancia};
use crate::paso3::PROB_INTERCAMBIO_CUOTAS;
use crate::tabu::DEFAULT_ROW_SEQUENCE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modelo {
    Paso1,
    Paso3,
}

/// Una fila de la tabla de instancias del brief.
#[derive(Clone, Copy, Debug)]
pub struct Especificacion {
    pub id: &'static str,
    pub modelo: Modelo,
    pub columnas: usize,
    pub filas: usize,
    /// round(0.2·|V|) existentes al azar.
    pub existentes: bool,
    /// n_j = round(0.9·C⁺_j): sobran libres vacías.
    pub huecos: bool,
}

const fn espec(
    id: &'static str,
    modelo: Modelo,
    lado: (usize, usize),
    existentes: bool,
    huecos: bool,
) -> Especificacion {
    Especificacion {
        id,
        modelo,
        columnas: lado.0,
        filas: lado.1,
        existentes,
        huecos,
    }
}

/// Las 10 instancias de validación (mallas `Completa`).
pub const ESPECIFICACIONES: [Especificacion; 10] = [
    espec("p3_a", Modelo::Paso3, (7, 7), false, false),
    espec("p3_b", Modelo::Paso3, (9, 9), false, false),
    espec("p3_c", Modelo::Paso3, (7, 7), true, false),
    espec("p3_d", Modelo::Paso3, (8, 8), true, false),
    espec("p3_e", Modelo::Paso3, (10, 10), true, false),
    espec("p3_f", Modelo::Paso3, (8, 8), true, true),
    espec("p3_g", Modelo::Paso3, (10, 10), true, true),
    espec("p1_a", Modelo::Paso1, (7, 7), false, false),
    espec("p1_b", Modelo::Paso1, (8, 8), false, false),
    espec("p1_c", Modelo::Paso1, (10, 10), false, false),
];

/// Formato de `inst_<id>.json`. Especies 0..9; −1 = libre (en `fija`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstanciaVal {
    pub id: String,
    pub modelo: Modelo,
    pub columnas: usize,
    pub filas: usize,
    pub disposicion: String,
    pub n_pos: usize,
    /// [x, y] de cada posición (m).
    pub posiciones: Vec<[f64; 2]>,
    /// [u, v, anillo] con u < v, en el orden en que `z_total` recorre los pares.
    pub aristas: Vec<(usize, usize, u16)>,
    /// peso[anillo][j][k] = `tabla.peso(anillo, j + 1, k + 1)` (w·h exacto de Rust).
    pub peso: Vec<Vec<Vec<f64>>>,
    /// Especie de la existente en cada posición; −1 = libre.
    pub fija: Vec<i32>,
    /// Paso 3: n_j (tope de nuevas por especie en las libres); paso 1: null.
    pub cuotas: Option<Vec<usize>>,
    /// Paso 3: plantas nuevas M; paso 1: null.
    pub m: Option<usize>,
    pub banda_l: Vec<usize>,
    pub banda_u: Vec<usize>,
    /// Paso 1: especies con L_j < U_j; paso 3: null.
    pub n_especies_libres: Option<usize>,
    /// Paso 1: D_tot = |V|; paso 3: null.
    pub d_tot: Option<usize>,
    /// θ_j (paso 1); ceros en paso 3.
    pub theta: Vec<f64>,
    /// γ* en paso 1; 1 en paso 3.
    pub gamma: f64,
    pub semilla_existentes: u64,
    /// Z entre pares existente-existente (constante del paso 3; 0 en paso 1).
    pub objetivo_constante: f64,
    /// Composición nominal D (mayor residuo) de |V|.
    pub d_nominal: Vec<usize>,
}

/// Malla, vecindad, grafo y tabla de competencia de un rectángulo `Completa`, con los mismos
/// anillos, h y W que `paso1::Instancia::nueva`.
pub struct Red {
    pub malla: Malla,
    pub vecindad: Vecindad,
    pub grafo: Grafo,
    pub tabla: TablaCompetencia,
}

pub fn red(columnas: usize, filas: usize, datos: &Datos, config: &Config) -> Red {
    let malla = Malla::rectangular(columnas, filas, Disposicion::Completa, config.d);
    let r = radios(&datos.especies, config.d, config.factor_radios);
    let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
    let w = matriz_w(&datos.especies, config.kappa);
    let tabla = TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia);
    Red {
        grafo: Grafo::desde(&vecindad),
        malla,
        vecindad,
        tabla,
    }
}

/// Pares {u, v} (u < v) en el orden de `z_total`: u creciente y los vecinos en el orden del grafo.
fn aristas(grafo: &Grafo, n_pos: usize) -> Vec<(usize, usize, u16)> {
    let mut aristas = Vec::new();
    for u in 0..n_pos {
        for vecino in grafo.vecinos(u) {
            let v = vecino.pos as usize;
            if v > u {
                aristas.push((u, v, vecino.anillo));
            }
        }
    }
    aristas
}

fn pesos(tabla: &TablaCompetencia) -> Vec<Vec<Vec<f64>>> {
    (0..tabla.n_anillos())
        .map(|a| {
            (0..N_ESPECIES)
                .map(|j| {
                    (0..N_ESPECIES)
                        .map(|k| tabla.peso(a as u16, j as u8 + 1, k as u8 + 1))
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// ¿Las aristas y pesos del JSON son los que da la configuración actual?
pub fn coincide(inst: &InstanciaVal, red: &Red) -> bool {
    red.malla.len() == inst.n_pos
        && aristas(&red.grafo, inst.n_pos) == inst.aristas
        && pesos(&red.tabla) == inst.peso
}

/// T existentes en posiciones distintas al azar con especies ~ Multinomial(T, p̂), con el mismo
/// orden de sorteo que `escenarios::generar` (T sorteos categóricos, luego las posiciones).
pub fn existentes(
    n_pos: usize,
    t: usize,
    p_hat: &[f64; N_ESPECIES],
    semilla: u64,
) -> Vec<(usize, u8)> {
    let mut rng = ChaCha8Rng::seed_from_u64(semilla);
    let total: f64 = p_hat.iter().sum();
    let mut x = [0usize; N_ESPECIES];
    for _ in 0..t {
        let u = rng.gen::<f64>() * total;
        let mut acumulado = 0.0;
        let mut elegida = N_ESPECIES - 1;
        for (j, p) in p_hat.iter().enumerate() {
            acumulado += p;
            if u < acumulado {
                elegida = j;
                break;
            }
        }
        x[elegida] += 1;
    }
    let mut it = sample(&mut rng, n_pos, t).into_iter();
    let mut posiciones = Vec::with_capacity(t);
    for (j, &n) in x.iter().enumerate() {
        for _ in 0..n {
            posiciones.push((it.next().expect("hay T posiciones"), j as u8));
        }
    }
    posiciones
}

pub fn construir(
    espec: &Especificacion,
    datos: &Datos,
    config: &Config,
    mc: &ParamsMc,
) -> InstanciaVal {
    let r = red(espec.columnas, espec.filas, datos, config);
    let n_pos = r.malla.len();
    let deltas: Vec<u32> = datos.especies.iter().map(|e| e.delta).collect();
    let d_nominal = composicion_nominal(n_pos, &deltas);
    let b = banda(&d_nominal, config.epsilon);
    let mut fija = vec![-1i32; n_pos];
    let t = if espec.existentes {
        (0.2 * n_pos as f64).round() as usize
    } else {
        0
    };
    for (u, j) in existentes(n_pos, t, &mc.p_hat, config.semilla) {
        fija[u] = j as i32;
    }
    let aristas = aristas(&r.grafo, n_pos);
    let peso = pesos(&r.tabla);
    let objetivo_constante = aristas
        .iter()
        .filter(|&&(u, v, _)| fija[u] >= 0 && fija[v] >= 0)
        .fold(0.0, |z, &(u, v, a)| {
            z + peso[a as usize][fija[u] as usize][fija[v] as usize]
        });
    let mut inst = InstanciaVal {
        id: espec.id.to_string(),
        modelo: espec.modelo,
        columnas: espec.columnas,
        filas: espec.filas,
        disposicion: "Completa".to_string(),
        n_pos,
        posiciones: r.malla.posiciones.iter().map(|p| [p.x, p.y]).collect(),
        aristas,
        peso,
        fija,
        cuotas: None,
        m: None,
        banda_l: b.l.clone(),
        banda_u: b.u.clone(),
        n_especies_libres: None,
        d_tot: None,
        theta: vec![0.0; N_ESPECIES],
        gamma: 1.0,
        semilla_existentes: config.semilla,
        objetivo_constante,
        d_nominal,
    };
    match espec.modelo {
        Modelo::Paso3 => {
            let mut x = [0usize; N_ESPECIES];
            for &f in inst.fija.iter().filter(|&&f| f >= 0) {
                x[f as usize] += 1;
            }
            // C⁺ = (D − X)⁺; sin existentes C⁺ = D.
            let c: Vec<usize> = (0..N_ESPECIES)
                .map(|j| inst.d_nominal[j].saturating_sub(x[j]))
                .collect();
            let n: Vec<usize> = if espec.huecos {
                c.iter()
                    .map(|&c| (0.9 * c as f64).round() as usize)
                    .collect()
            } else {
                c
            };
            inst.m = Some(n.iter().sum::<usize>().min(n_pos - t));
            inst.cuotas = Some(n);
        }
        Modelo::Paso1 => {
            inst.d_tot = Some(n_pos);
            inst.gamma = config.gamma_star;
            inst.theta = theta(&datos.especies, config.m_escala, config.usar_s2).to_vec();
            inst.n_especies_libres = Some((0..N_ESPECIES).filter(|&j| b.l[j] < b.u[j]).count());
        }
    }
    inst
}

/// (Z, objetivo) de φ con los datos del JSON: Z = Σ peso de las aristas con ambos extremos
/// plantados (incluye pares existente-existente; las vacías no pesan), sumado en el orden de
/// `z_total`; objetivo = γ·Z + Σ θ (como `Problema::objetivo`), que en paso 3 es Z.
pub fn evaluar(inst: &InstanciaVal, phi: &[u8]) -> (f64, f64) {
    let mut z = 0.0;
    for &(u, v, a) in &inst.aristas {
        let (p, q) = (phi[u], phi[v]);
        if p > 0 && q > 0 {
            z += inst.peso[a as usize][p as usize - 1][q as usize - 1];
        }
    }
    let mortalidad: f64 = phi
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| inst.theta[c as usize - 1])
        .sum();
    (z, inst.gamma * z + mortalidad)
}

/// Restricciones que φ no cumple (vacío = factible): longitud, existentes en su lugar, y
/// cuotas con M nuevas (paso 3) o banda con D_tot plantas (paso 1).
pub fn violaciones(inst: &InstanciaVal, phi: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    if phi.len() != inst.n_pos {
        v.push(format!(
            "φ tiene {} posiciones y la malla {}",
            phi.len(),
            inst.n_pos
        ));
        return v;
    }
    if let Some(u) = phi.iter().position(|&c| c as usize > N_ESPECIES) {
        v.push(format!(
            "posición {u}: especie {} fuera de 0..9",
            phi[u] as i32 - 1
        ));
        return v;
    }
    let mut cuenta = [0usize; N_ESPECIES];
    for (u, &c) in phi.iter().enumerate() {
        if inst.fija[u] >= 0 {
            if c as i32 != inst.fija[u] + 1 {
                v.push(format!(
                    "posición {u}: la existente {} cambió",
                    inst.fija[u]
                ));
            }
        } else if c > 0 {
            cuenta[c as usize - 1] += 1;
        }
    }
    let total: usize = cuenta.iter().sum();
    match inst.modelo {
        Modelo::Paso3 => {
            let n = inst.cuotas.as_deref().unwrap_or(&[]);
            for j in 0..N_ESPECIES {
                if cuenta[j] > n.get(j).copied().unwrap_or(0) {
                    v.push(format!(
                        "especie {j}: {} nuevas > n = {:?}",
                        cuenta[j],
                        n.get(j)
                    ));
                }
            }
            if Some(total) != inst.m {
                v.push(format!("{total} nuevas ≠ M = {:?}", inst.m));
            }
        }
        Modelo::Paso1 => {
            for j in 0..N_ESPECIES {
                if cuenta[j] < inst.banda_l[j] || cuenta[j] > inst.banda_u[j] {
                    v.push(format!(
                        "especie {j}: D = {} fuera de [{}, {}]",
                        cuenta[j], inst.banda_l[j], inst.banda_u[j]
                    ));
                }
            }
            if Some(total) != inst.d_tot {
                v.push(format!("{total} plantas ≠ D_tot = {:?}", inst.d_tot));
            }
        }
    }
    v
}

/// φ inicial del paso 3: existentes en su lugar y, recorriendo la malla en serpentina (filas
/// pares de izquierda a derecha, impares al revés), cada libre recibe la siguiente especie de
/// `DEFAULT_ROW_SEQUENCE` que aún tenga cuota, hasta plantar M; las libres restantes quedan vacías.
pub fn inicial_paso3(inst: &InstanciaVal, malla: &Malla) -> Vec<u8> {
    let n = inst
        .cuotas
        .as_deref()
        .expect("instancia del paso 3 con cuotas");
    let m = inst.m.expect("instancia del paso 3 con M");
    let mut phi: Vec<u8> = inst.fija.iter().map(|&f| (f + 1) as u8).collect();
    let mut cuenta = [0usize; N_ESPECIES];
    let (mut plantadas, mut k) = (0, 0);
    for fila in 0..malla.filas() {
        let mut posiciones: Vec<usize> = malla.fila(fila).collect();
        if fila % 2 == 1 {
            posiciones.reverse();
        }
        for u in posiciones {
            if inst.fija[u] >= 0 || plantadas == m {
                continue;
            }
            let siguiente = (0..DEFAULT_ROW_SEQUENCE.len())
                .map(|i| DEFAULT_ROW_SEQUENCE[(k + i) % DEFAULT_ROW_SEQUENCE.len()])
                .position(|j| cuenta[j] < n[j]);
            let j = match siguiente {
                Some(i) => {
                    let j = DEFAULT_ROW_SEQUENCE[(k + i) % DEFAULT_ROW_SEQUENCE.len()];
                    k += i + 1;
                    j
                }
                // Sólo si a todas las especies de la secuencia se les acabó la cuota.
                None => match (0..N_ESPECIES).find(|&j| cuenta[j] < n[j]) {
                    Some(j) => j,
                    None => continue,
                },
            };
            phi[u] = j as u8 + 1;
            cuenta[j] += 1;
            plantadas += 1;
        }
    }
    phi
}

/// Una corrida del tabú sobre una instancia de validación.
#[derive(Clone, Debug)]
pub struct CorridaVal {
    pub semilla: u64,
    /// Z del φ final (`evaluar`).
    pub z: f64,
    /// γ·Z + Σθ·D (paso 1) o Z (paso 3) del φ final.
    pub objetivo: f64,
    pub tiempo_s: f64,
    pub iteraciones: usize,
    /// Paso 1: `tope` si alguna de las dos búsquedas (banda y ajuste C) llegó al tope.
    pub motivo_paro: MotivoParo,
    pub phi: Vec<u8>,
}

/// Paso 3: `Restriccion::Cuotas` desde `inicial_paso3`. Paso 1: `resolver_paso1` (búsqueda en
/// la banda con γ y θ, luego ajuste C sólo con intercambios) sobre esta malla. Topes de
/// `tabu.params_para(semilla, |V|)`.
pub fn correr_tabu(inst: &InstanciaVal, red: &Red, tabu: &ConfigTabu, semilla: u64) -> CorridaVal {
    let params = tabu.params_para(semilla, inst.n_pos);
    let theta: [f64; N_ESPECIES] = std::array::from_fn(|j| inst.theta[j]);
    let (phi, tiempo_s, iteraciones, motivo_paro) = match inst.modelo {
        Modelo::Paso3 => {
            let problema = Problema {
                grafo: &red.grafo,
                tabla: &red.tabla,
                fija: inst.fija.iter().map(|&f| f >= 0).collect(),
                gamma: inst.gamma,
                theta,
                restriccion: Restriccion::Cuotas {
                    n: inst
                        .cuotas
                        .clone()
                        .expect("instancia del paso 3 con cuotas"),
                },
            };
            let params = ParamsTabu {
                prob_intercambio: PROB_INTERCAMBIO_CUOTAS,
                ..params
            };
            let r = buscar(&problema, inicial_paso3(inst, &red.malla), &params);
            (r.phi, r.tiempo_s, r.iteraciones, r.motivo_paro)
        }
        Modelo::Paso1 => {
            let instancia = Instancia {
                nombre: inst.id.clone(),
                map_id: 0,
                // `resolver_paso1` no usa el área.
                area: 0.0,
                malla: red.malla.clone(),
                d_tot: inst.d_tot.expect("instancia del paso 1 con D_tot"),
                nominal: inst.d_nominal.clone(),
                banda: Banda {
                    l: inst.banda_l.clone(),
                    u: inst.banda_u.clone(),
                },
                grafo: red.grafo.clone(),
                tabla: red.tabla.clone(),
                theta,
                costos: [0.0; N_ESPECIES],
            };
            let (fila, phi) = resolver_paso1(&instancia, inst.gamma, &params);
            let motivo = if fila.motivo_paro_paso1 == MotivoParo::SinMejora
                && fila.motivo_paro_opt == MotivoParo::SinMejora
            {
                MotivoParo::SinMejora
            } else {
                MotivoParo::Tope
            };
            (
                phi,
                fila.tiempo_paso1_s + fila.tiempo_opt_s,
                fila.iter_paso1 + fila.iter_opt,
                motivo,
            )
        }
    };
    let (z, objetivo) = evaluar(inst, &phi);
    CorridaVal {
        semilla,
        z,
        objetivo,
        tiempo_s,
        iteraciones,
        motivo_paro,
        phi,
    }
}

/// φ como especies separadas por `;` (−1 = vacía).
pub fn phi_a_texto(phi: &[u8]) -> String {
    phi.iter()
        .map(|&c| (c as i32 - 1).to_string())
        .collect::<Vec<_>>()
        .join(";")
}

/// Especie (−1..9) → código φ (0..10).
fn codigo(especie: i32) -> Result<u8, Box<dyn Error>> {
    if (-1..N_ESPECIES as i32).contains(&especie) {
        Ok((especie + 1) as u8)
    } else {
        Err(format!("especie {especie} fuera de −1..9").into())
    }
}

/// Asignaciones de un CSV: una por fila si tiene la columna `phi` (especies separadas por `;`,
/// como `tabu_<id>.csv`), o una sola con la columna `especie` por posición (como los patrones).
pub fn leer_asignaciones(path: &Path) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let enc = lector.headers()?.clone();
    let columna = |nombre: &str| enc.iter().position(|h| h.trim() == nombre);
    if let Some(c) = columna("phi") {
        let mut todas = Vec::new();
        for registro in lector.records() {
            let r = registro?;
            let phi = r[c]
                .split(';')
                .map(|s| codigo(s.trim().parse()?))
                .collect::<Result<Vec<u8>, _>>()?;
            todas.push(phi);
        }
        Ok(todas)
    } else if let Some(c) = columna("especie") {
        let mut phi = Vec::new();
        for registro in lector.records() {
            // Python puede escribir los enteros como -1.0.
            phi.push(codigo(registro?[c].trim().parse::<f64>()?.round() as i32)?);
        }
        Ok(vec![phi])
    } else {
        Err(format!("{}: falta la columna phi o especie", path.display()).into())
    }
}
