//! Paso 2: compra de plantas por PLE con escenarios Monte Carlo.
//!
//! Es una heurística, no el paso 2 oficial: el oficial es `python/paso2_compra.py` (PuLP/HiGHS).
//! Difiere de PuLP en 34 de 93 filas, con un objetivo ligeramente peor.
//!
//! Puerto de `python/paso2_compra.py`. Con D* fija, `C_jω = D*_j − X_jω` es la necesidad de la
//! especie j en el escenario ω. Se compra un `q_j ≥ 0` entero por especie que minimiza
//!
//! ```text
//! Σ_j c_j·q_j  +  (1/n)·Σ_ω Σ_j [π_j·faltante_jω + η·sobrante_jω]
//! ```
//! sujeto a `Σ_j c_j·q_j ≤ presupuesto`, con `π_j = π_factor·c_j`. El término `η` penaliza
//! comprar de más (0 por defecto, como en el voceador).
//!
//! El objetivo separa por especie y sólo las une el presupuesto, así que se resuelve casi
//! exacto: se fija un precio λ ≥ 0 de la restricción, cada especie se resuelve sola
//! minimizando `costo_j(q_j) + λ·c_j·q_j` (barrido sobre enteros con prefijos), se hace
//! bisectión en λ hasta `Σ c_j·q_j ≤ presupuesto`, y una reparación gasta la holgura en
//! unidades marginales net-positivas (λ > 0 sobreprecifica y deja presupuesto sin gastar).

use std::error::Error;

use crate::datos::N_ESPECIES;

/// Entrada del PLE.
pub struct EntradaPaso2<'a> {
    /// D* (o D̄ nominal) por especie.
    pub d: &'a [u32; N_ESPECIES],
    /// X de cada escenario, uno por fila.
    pub x: &'a [[u32; N_ESPECIES]],
    /// Costo unitario c_j de la planta de la especie j.
    pub c: &'a [f64; N_ESPECIES],
    /// Peso del faltante: π_j = `pi_factor`·c_j.
    pub pi: &'a [f64; N_ESPECIES],
    /// Presupuesto B del polígono; None = sin restricción.
    pub presupuesto: Option<f64>,
    /// Peso η del sobrante (0 por defecto).
    pub eta: f64,
}

/// Lo que el cálculo por especie necesita: costos, pesos y necesidades ordenadas por especie.
struct DatosEspecie<'a> {
    c: &'a [f64; N_ESPECIES],
    pi: &'a [f64; N_ESPECIES],
    eta: f64,
    /// Necesidades de la especie j ordenadas ascendentemente.
    ordenadas: &'a [Vec<f64>; N_ESPECIES],
    /// Prefijos `prefes[j][k] = Σ_{i<k} ordenadas[j][i]` (longitud n+1).
    prefijos: &'a [Vec<f64>; N_ESPECIES],
    c_max: [u32; N_ESPECIES],
}

/// Resultado del paso 2 para una instancia.
#[derive(Clone, Debug)]
pub struct FilaPaso2 {
    /// Cantidad comprada por especie.
    pub q: [u32; N_ESPECIES],
    /// Σ_j c_j·q_j.
    pub costo: f64,
    /// Presupuesto usado; None si no había restricción.
    pub presupuesto: Option<f64>,
    pub cabe: bool,
    /// Media de Σ_j (D*_j − X_jω − q_j)⁺ sobre los escenarios.
    pub faltante_esperado: f64,
    /// Media de Σ_j (q_j − (D*_j − X_jω))⁺ sobre los escenarios.
    pub sobrante_esperado: f64,
    /// Σ_j c_j·q_j + (1/n)·Σ_ω Σ_j [π_j·faltante + η·sobrante].
    pub objetivo: f64,
    /// Regla del voceador sin presupuesto, para comparar.
    pub q_voceador: [u32; N_ESPECIES],
    pub costo_voceador: f64,
}

/// Necesidad por escenario: `C = D* − X`.
pub fn necesidad(d: &[u32; N_ESPECIES], x: &[u32; N_ESPECIES]) -> [u32; N_ESPECIES] {
    std::array::from_fn(|j| d[j].saturating_sub(x[j]))
}

/// Regla del voceador: cuantil empírico de nivel `(π_j − c_j)/(π_j + η)` de `C_j`, entero y ≥ 0.
pub fn voceador(
    c: &[f64; N_ESPECIES],
    pi: &[f64; N_ESPECIES],
    eta: f64,
    d: &[u32; N_ESPECIES],
    x: &[[u32; N_ESPECIES]],
) -> [u32; N_ESPECIES] {
    let n = x.len().max(1);
    std::array::from_fn(|j| {
        let nivel = (pi[j] - c[j]) / (pi[j] + eta);
        let k = ((n as f64 * nivel - 1e-12).ceil().max(1.0) as usize).min(n) - 1;
        let mut valores: Vec<u32> = x.iter().map(|fila| necesidad(d, fila)[j]).collect();
        valores.sort_unstable();
        (valores[k] as f64 - 1e-9).ceil().max(0.0) as u32
    })
}

/// Costo esperado de la especie j si se compra `q`: `c_j·q + media(π_j·faltante + η·sobrante)`.
/// Con las necesidades ordenadas y sus prefijos es O(log n) por consulta.
fn costo_esperado(q: f64, c: f64, pi: f64, eta: f64, ordenadas: &[f64], prefijos: &[f64]) -> f64 {
    let n = ordenadas.len().max(1) as f64;
    // Primer índice con necesidad > q: los de arriba aportan π·(cn−q), los de abajo η·(q−cn).
    let k = ordenadas.partition_point(|&cn| cn <= q);
    let n_abajo = k as f64;
    let suma_abajo = prefijos[k];
    let n_arriba = n - n_abajo;
    let suma_arriba = prefijos[ordenadas.len()] - suma_abajo;
    let faltante = suma_arriba - q * n_arriba;
    let sobrante = q * n_abajo - suma_abajo;
    c * q + (pi * faltante + eta * sobrante) / n
}

/// Mejor `q_j` para un precio λ de la restricción de presupuesto.
/// En empates de valor se prefiere el `q` mayor (gasta presupuesto que reduce el faltante).
fn mejor_q(datos: &DatosEspecie, j: usize, lambda: f64) -> u32 {
    // Con η = 0 el sobrante no se penaliza: el óptimo nunca pasa de la necesidad máxima.
    let tope = if datos.eta > 0.0 {
        datos.c_max[j] + (datos.pi[j] / datos.eta).ceil() as u32 + 1
    } else {
        datos.c_max[j]
    };
    let mut mejor_q = 0u32;
    let mut mejor_v = costo_esperado(
        0.0,
        datos.c[j],
        datos.pi[j],
        datos.eta,
        &datos.ordenadas[j],
        &datos.prefijos[j],
    );
    for q in 1..=tope {
        let v = costo_esperado(
            q as f64,
            datos.c[j],
            datos.pi[j],
            datos.eta,
            &datos.ordenadas[j],
            &datos.prefijos[j],
        ) + lambda * datos.c[j] * q as f64;
        if v < mejor_v - 1e-9 || (v - mejor_v).abs() <= 1e-9 {
            mejor_v = v.min(mejor_v);
            mejor_q = q;
        }
    }
    mejor_q
}

/// `q` de todas las especies y su costo de compra para un precio λ.
fn solucion_para_lambda(datos: &DatosEspecie, lambda: f64) -> ([u32; N_ESPECIES], f64) {
    let mut q = [0u32; N_ESPECIES];
    let mut costo = 0.0;
    for j in 0..N_ESPECIES {
        q[j] = mejor_q(datos, j, lambda);
        costo += datos.c[j] * q[j] as f64;
    }
    (q, costo)
}

/// Cambio del objetivo al comprar una unidad más de la especie j en `q`: `c_j + Δpenalización`.
/// Negativo ⇒ la unidad reduce el objetivo esperado.
fn cambio_marginal(datos: &DatosEspecie, j: usize, q: u32) -> (f64, f64) {
    let n = datos.ordenadas[j].len().max(1) as f64;
    let k = datos.ordenadas[j].partition_point(|&cn| cn <= q as f64) as f64;
    let delta_penal = (datos.eta * k - datos.pi[j] * (n - k)) / n;
    (datos.c[j], datos.c[j] + delta_penal)
}

/// Reparación: con el presupuesto aún no gastado, compra unidades de mayor beneficio
/// neto mientras quepan en el presupuesto. Cierra el hueco que la bisectión en λ deja
/// (λ > 0 sobreprecifica y no gasta el presupuesto completo aunque convenga).
fn reparar_presupuesto(
    datos: &DatosEspecie,
    q: &mut [u32; N_ESPECIES],
    costo: &mut f64,
    presupuesto: f64,
) {
    let mut remanente = presupuesto - *costo;
    loop {
        if remanente <= 1e-9 {
            return;
        }
        let mut mejor_j: Option<usize> = None;
        let mut mejor_cambio = 0.0f64; // >=0: no conviene
        for j in 0..N_ESPECIES {
            let c_u = datos.c[j];
            if c_u > remanente + 1e-9 {
                continue;
            }
            let (_, cambio) = cambio_marginal(datos, j, q[j]);
            if cambio < mejor_cambio - 1e-12 {
                mejor_cambio = cambio;
                mejor_j = Some(j);
            }
        }
        match mejor_j {
            Some(j) => {
                *costo += datos.c[j];
                q[j] += 1;
                remanente -= datos.c[j];
            }
            None => return,
        }
    }
}

/// (costo de compra, faltante medio, sobrante medio, penalización esperada) de un `q` dado.
/// La penalización es `(1/n)·Σ_ω Σ_j [π_j·faltante_jω + η·sobrante_jω]`, con peso por especie.
fn metricas(
    c: &[f64; N_ESPECIES],
    pi: &[f64; N_ESPECIES],
    eta: f64,
    d: &[u32; N_ESPECIES],
    x: &[[u32; N_ESPECIES]],
    q: &[u32; N_ESPECIES],
) -> (f64, f64, f64, f64) {
    let n = x.len().max(1) as f64;
    let (mut faltante, mut sobrante, mut penal) = (0.0, 0.0, 0.0);
    for fila in x {
        let c_nec = necesidad(d, fila);
        for j in 0..N_ESPECIES {
            let f = (c_nec[j] as f64 - q[j] as f64).max(0.0);
            let s = (q[j] as f64 - c_nec[j] as f64).max(0.0);
            faltante += f;
            sobrante += s;
            penal += pi[j] * f + eta * s;
        }
    }
    let costo: f64 = (0..N_ESPECIES).map(|j| c[j] * q[j] as f64).sum();
    (costo, faltante / n, sobrante / n, penal / n)
}

/// Resuelve el PLE por bisectión en el precio de la restricción de presupuesto,
/// seguida de una reparación que gasta la holgura en unidades net-positivas.
pub fn resolver_ple(entrada: &EntradaPaso2) -> Result<FilaPaso2, Box<dyn Error>> {
    let mut ordenadas: [Vec<f64>; N_ESPECIES] = std::array::from_fn(|_| Vec::new());
    let mut c_max = [0u32; N_ESPECIES];
    for fila in entrada.x {
        let cn = necesidad(entrada.d, fila);
        for j in 0..N_ESPECIES {
            ordenadas[j].push(cn[j] as f64);
            c_max[j] = c_max[j].max(cn[j]);
        }
    }
    let mut prefijos: [Vec<f64>; N_ESPECIES] = std::array::from_fn(|_| Vec::new());
    for j in 0..N_ESPECIES {
        ordenadas[j].sort_unstable_by(|a, b| a.partial_cmp(b).expect("f64 sin NaN"));
        let mut acumulado = 0.0;
        prefijos[j].push(0.0);
        for &cn in &ordenadas[j] {
            acumulado += cn;
            prefijos[j].push(acumulado);
        }
    }
    let datos = DatosEspecie {
        c: entrada.c,
        pi: entrada.pi,
        eta: entrada.eta,
        ordenadas: &ordenadas,
        prefijos: &prefijos,
        c_max,
    };

    let (mut q, mut costo) = solucion_para_lambda(&datos, 0.0);
    if let Some(b) = entrada.presupuesto {
        if costo > b + 1e-9 {
            // Bisectión en λ: λ creciente ⇒ costo de compra decreciente. Se mantiene
            // `lo` infraviola (costo > b) y `hi` cumple (costo ≤ b), buscando el mayor
            // costo que aún cabe en el presupuesto.
            let mut lo = 0.0;
            let mut hi = 1.0;
            while solucion_para_lambda(&datos, hi).1 > b + 1e-9 && hi < 1e12 {
                hi *= 2.0;
            }
            let mut mejor = solucion_para_lambda(&datos, hi);
            for _ in 0..150 {
                let mid = (lo + hi) / 2.0;
                let (q_m, costo_m) = solucion_para_lambda(&datos, mid);
                if costo_m <= b + 1e-9 {
                    hi = mid;
                    mejor = (q_m, costo_m);
                } else {
                    lo = mid;
                }
            }
            (q, costo) = mejor;
        }
        reparar_presupuesto(&datos, &mut q, &mut costo, b);
    }
    let cabe = entrada.presupuesto.is_none_or(|b| costo <= b + 1e-6);

    let (costo, faltante, sobrante, penal) =
        metricas(entrada.c, entrada.pi, entrada.eta, entrada.d, entrada.x, &q);
    let q_v = voceador(entrada.c, entrada.pi, entrada.eta, entrada.d, entrada.x);
    let costo_v: f64 = (0..N_ESPECIES).map(|j| entrada.c[j] * q_v[j] as f64).sum();
    let objetivo = costo + penal;

    Ok(FilaPaso2 {
        q,
        costo,
        presupuesto: entrada.presupuesto,
        cabe,
        faltante_esperado: faltante,
        sobrante_esperado: sobrante,
        objetivo,
        q_voceador: q_v,
        costo_voceador: costo_v,
    })
}
