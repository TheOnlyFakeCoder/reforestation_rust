//! Escenarios Monte Carlo de las plantas existentes y sus resúmenes.
//!
//! Rust es la única fuente de escenarios: el modelo de compra (Python) y el ajuste al polígono
//! real regeneran los mismos X llamando a `generar` con la misma semilla.

use std::error::Error;
use std::path::Path;

use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::Deserialize;

use crate::datos::N_ESPECIES;

/// Parámetros del ajuste (`results/mc_params.json`, escritos por Python).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ParamsMc {
    pub mu: f64,
    pub sigma: f64,
    pub p_hat: [f64; N_ESPECIES],
}

impl ParamsMc {
    pub fn leer(path: &Path) -> Result<ParamsMc, Box<dyn Error>> {
        let texto =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", path.display()))?)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Escenario {
    pub omega: usize,
    /// Densidad sorteada ρ (plantas/ha).
    pub rho: f64,
    /// Total de plantas existentes T = round(ρ·A), acotado por |V|.
    pub t: usize,
    pub x: [u32; N_ESPECIES],
    /// (posición de la malla, especie) de cada planta existente.
    pub posiciones: Vec<(usize, u8)>,
}

/// Escenario ω del polígono con `map_id` (0 para 1 ha). Orden de sorteo fijo:
/// ρ ~ Normal(μ, σ), T = round(ρ·A), X ~ Multinomial(T, p̂) con T sorteos categóricos y
/// T posiciones distintas de la malla, asignadas en orden a las especies (X_0 primero).
pub fn generar(
    n_posiciones: usize,
    area: f64,
    map_id: u32,
    omega: usize,
    semilla: u64,
    params: &ParamsMc,
) -> Escenario {
    let mut rng = ChaCha8Rng::seed_from_u64(
        semilla
            ^ (map_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (omega as u64).wrapping_mul(0xD1B5_4A32_D192_ED03),
    );
    let normal = Normal::new(params.mu, params.sigma).expect("sigma debe ser finita y ≥ 0");
    let rho = normal.sample(&mut rng);
    let t = ((rho * area).round().max(0.0) as usize).min(n_posiciones);

    let total: f64 = params.p_hat.iter().sum();
    let mut x = [0u32; N_ESPECIES];
    for _ in 0..t {
        let u = rng.gen::<f64>() * total;
        let mut acumulado = 0.0;
        let mut elegida = N_ESPECIES - 1;
        for (j, p) in params.p_hat.iter().enumerate() {
            acumulado += p;
            if u < acumulado {
                elegida = j;
                break;
            }
        }
        x[elegida] += 1;
    }

    let indices = sample(&mut rng, n_posiciones, t);
    let mut posiciones = Vec::with_capacity(t);
    let mut it = indices.into_iter();
    for (j, &n) in x.iter().enumerate() {
        for _ in 0..n {
            posiciones.push((it.next().expect("hay T posiciones"), j as u8));
        }
    }
    Escenario { omega, rho, t, x, posiciones }
}

/// Media y desviación estándar muestral (n − 1).
pub fn estadisticos(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let media = v.iter().sum::<f64>() / n;
    if v.len() < 2 {
        return (media, 0.0);
    }
    let var = v.iter().map(|x| (x - media).powi(2)).sum::<f64>() / (n - 1.0);
    (media, var.sqrt())
}

/// Percentil `p` ∈ [0, 1] con interpolación lineal (como `numpy.percentile`).
pub fn percentil(v: &[f64], p: f64) -> f64 {
    let mut ord = v.to_vec();
    ord.sort_by(|a, b| a.total_cmp(b));
    let pos = p * (ord.len() - 1) as f64;
    let (i, f) = (pos.floor() as usize, pos - pos.floor());
    if i + 1 < ord.len() {
        ord[i] + f * (ord[i + 1] - ord[i])
    } else {
        ord[i]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilaEspecie {
    pub especie: usize,
    pub d: u32,
    pub e_c: f64,
    pub sd_c: f64,
    pub ic95_inf: f64,
    pub ic95_sup: f64,
    pub p90: f64,
    pub p95: f64,
    /// Réplicas para media con error ±0.5 plantas al 95 %: ceil((1.96·s/0.5)²).
    pub n_necesario: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilaCosto {
    pub costo_esperado: f64,
    pub costo_sd: f64,
    pub presupuesto: f64,
    pub p_costo_mayor_b: f64,
    pub violaciones_x_mayor_l: usize,
    pub escenarios_c_negativo: usize,
    pub especies_c_negativo: usize,
}

/// Resumen de C_j = D_j − X_j y del costo Σ c_j·C⁺_j sobre los escenarios.
pub fn resumir(
    escenarios: &[Escenario],
    d: &[u32; N_ESPECIES],
    l: &[u32; N_ESPECIES],
    costos: &[f64; N_ESPECIES],
    presupuesto: f64,
) -> (Vec<FilaEspecie>, FilaCosto) {
    let n = escenarios.len() as f64;
    let especies = (0..N_ESPECIES)
        .map(|j| {
            let c: Vec<f64> = escenarios.iter().map(|e| d[j] as f64 - e.x[j] as f64).collect();
            let (media, sd) = estadisticos(&c);
            let mitad = 1.96 * sd / n.sqrt();
            FilaEspecie {
                especie: j,
                d: d[j],
                e_c: media,
                sd_c: sd,
                ic95_inf: media - mitad,
                ic95_sup: media + mitad,
                p90: percentil(&c, 0.90),
                p95: percentil(&c, 0.95),
                n_necesario: ((1.96 * sd / 0.5).powi(2)).ceil() as u64,
            }
        })
        .collect();
    let costo: Vec<f64> = escenarios
        .iter()
        .map(|e| {
            (0..N_ESPECIES).map(|j| costos[j] * (d[j] as f64 - e.x[j] as f64).max(0.0)).sum()
        })
        .collect();
    let (costo_esperado, costo_sd) = estadisticos(&costo);
    let mut viol = 0;
    let mut esc_neg = 0;
    let mut sp_neg = [false; N_ESPECIES];
    for e in escenarios {
        let mut alguna = false;
        for j in 0..N_ESPECIES {
            viol += (e.x[j] > l[j]) as usize;
            if e.x[j] > d[j] {
                alguna = true;
                sp_neg[j] = true;
            }
        }
        esc_neg += alguna as usize;
    }
    let fila = FilaCosto {
        costo_esperado,
        costo_sd,
        presupuesto,
        p_costo_mayor_b: costo.iter().filter(|&&c| c > presupuesto).count() as f64 / n,
        violaciones_x_mayor_l: viol,
        escenarios_c_negativo: esc_neg,
        especies_c_negativo: sp_neg.iter().filter(|&&b| b).count(),
    };
    (especies, fila)
}
