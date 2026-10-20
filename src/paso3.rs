//! Paso 3: ajuste al polígono real. Dadas las cantidades compradas q (paso 2) y un escenario ω de
//! plantas existentes (fijas), la búsqueda tabú coloca las nuevas plantas en las posiciones libres
//! minimizando la competencia Z.

use std::error::Error;
use std::path::Path;

use crate::busqueda_tabu::{buscar, MotivoParo, ParamsTabu, Problema, Restriccion};
use crate::datos::N_ESPECIES;
use crate::escenarios::{generar, Escenario, ParamsMc};
use crate::objetivo::z_total;
use crate::paso1::Instancia;

/// Probabilidad de intercambio con `Restriccion::Cuotas`: sobre todo intercambios (spec §8).
pub const PROB_INTERCAMBIO_CUOTAS: f64 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Caso {
    /// Σn = |V_lib|.
    Usual,
    /// Σn < |V_lib|: sobran posiciones libres vacías.
    PresupuestoCorto,
    /// Σn > |V_lib|: las existentes ocupan lugar de sobra; M = |V_lib|.
    ExistentesDeSobra,
}

impl Caso {
    pub fn como_str(self) -> &'static str {
        match self {
            Caso::Usual => "usual",
            Caso::PresupuestoCorto => "presupuesto_corto",
            Caso::ExistentesDeSobra => "existentes_de_sobra",
        }
    }
}

/// n_j = min(q_j, C⁺_j) con C_j = D_j − X_j; M = min(|V_lib|, Σn); y el caso.
pub fn cuotas(
    q: &[u32; N_ESPECIES],
    d: &[u32; N_ESPECIES],
    x: &[u32; N_ESPECIES],
    v_lib: usize,
) -> (Vec<usize>, usize, Caso) {
    let n: Vec<usize> = (0..N_ESPECIES)
        .map(|j| q[j].min(d[j].saturating_sub(x[j])) as usize)
        .collect();
    let suma: usize = n.iter().sum();
    let caso = match suma.cmp(&v_lib) {
        std::cmp::Ordering::Equal => Caso::Usual,
        std::cmp::Ordering::Less => Caso::PresupuestoCorto,
        std::cmp::Ordering::Greater => Caso::ExistentesDeSobra,
    };
    (n, suma.min(v_lib), caso)
}

/// φ inicial: patrón base con las existentes encima (fijas); las libres se reparan a cuotas:
/// las especies sobre n_j sueltan plantas (quedan vacías) y los faltantes hasta M se plantan,
/// de la especie con más faltante, en posiciones libres vacías. Devuelve (φ, fija).
pub fn construir_inicial(
    base: &[u8],
    existentes: &[(usize, u8)],
    n: &[usize],
    m: usize,
) -> (Vec<u8>, Vec<bool>) {
    let mut phi = base.to_vec();
    let mut fija = vec![false; phi.len()];
    for &(u, j) in existentes {
        phi[u] = j + 1;
        fija[u] = true;
    }
    let mut cuenta = [0usize; N_ESPECIES + 1];
    for u in 0..phi.len() {
        if fija[u] || phi[u] == 0 {
            continue;
        }
        let j = phi[u] as usize;
        if cuenta[j] >= n[j - 1] {
            phi[u] = 0;
        } else {
            cuenta[j] += 1;
        }
    }
    let mut total: usize = cuenta[1..].iter().sum();
    let mut vacias = (0..phi.len())
        .filter(|&u| !fija[u] && phi[u] == 0)
        .collect::<Vec<_>>();
    vacias.reverse();
    while total < m {
        let Some(j) = (0..N_ESPECIES)
            .filter(|&j| cuenta[j + 1] < n[j])
            .max_by_key(|&j| n[j] - cuenta[j + 1])
        else {
            break;
        };
        let Some(u) = vacias.pop() else { break };
        phi[u] = j as u8 + 1;
        cuenta[j + 1] += 1;
        total += 1;
    }
    (phi, fija)
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilaPaso3 {
    pub instancia: String,
    pub map_id: u32,
    pub omega: usize,
    pub caso: Caso,
    pub t: usize,
    pub v_lib: usize,
    pub m: usize,
    pub z: f64,
    /// Z entre las plantas del polígono (existentes T + nuevas M).
    pub z_por_planta: f64,
    pub tiempo_s: f64,
    pub iteraciones: usize,
    pub motivo_paro: MotivoParo,
}

/// Resuelve el escenario ω: (fila, φ final con existentes y nuevas).
pub fn resolver_escenario(
    instancia: &Instancia,
    base: &[u8],
    q: &[u32; N_ESPECIES],
    d: &[u32; N_ESPECIES],
    escenario: &Escenario,
    params: &ParamsTabu,
) -> (FilaPaso3, Vec<u8>) {
    let v_lib = instancia.malla.len() - escenario.t;
    let (n, m, caso) = cuotas(q, d, &escenario.x, v_lib);
    let (inicial, fija) = construir_inicial(base, &escenario.posiciones, &n, m);
    let problema = Problema {
        grafo: &instancia.grafo,
        tabla: &instancia.tabla,
        fija,
        gamma: 1.0,
        theta: [0.0; N_ESPECIES],
        restriccion: Restriccion::Cuotas { n },
    };
    let params = ParamsTabu {
        prob_intercambio: PROB_INTERCAMBIO_CUOTAS,
        ..*params
    };
    let r = buscar(&problema, inicial, &params);
    let plantas = escenario.t + m;
    let z = z_total(&r.phi, &instancia.grafo, &instancia.tabla);
    (
        FilaPaso3 {
            instancia: instancia.nombre.clone(),
            map_id: instancia.map_id,
            omega: escenario.omega,
            caso,
            t: escenario.t,
            v_lib,
            m,
            z,
            z_por_planta: if plantas > 0 { z / plantas as f64 } else { 0.0 },
            tiempo_s: r.tiempo_s,
            iteraciones: r.iteraciones,
            motivo_paro: r.motivo_paro,
        },
        r.phi,
    )
}

/// Datos de entrada de una instancia para el paso 3.
pub struct EntradaPaso3<'a> {
    pub instancia: &'a Instancia,
    /// φ* del paso 1.
    pub base: &'a [u8],
    pub q: [u32; N_ESPECIES],
    pub d: [u32; N_ESPECIES],
    pub mc: &'a ParamsMc,
    pub semilla: u64,
}

/// Corre ω = 0..n_omega repartidos en hilos; el resultado no depende del número de hilos.
/// Devuelve las filas, el φ final y el escenario del ω = 0.
pub fn correr_instancia(
    entrada: &EntradaPaso3,
    n_omega: usize,
    params: &ParamsTabu,
) -> (Vec<FilaPaso3>, Vec<u8>, Escenario) {
    let inst = entrada.instancia;
    let hilos = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(n_omega.max(1));
    let mut salida: Vec<Option<(FilaPaso3, Vec<u8>)>> = vec![None; n_omega];
    let trozo = n_omega.div_ceil(hilos).max(1);
    std::thread::scope(|s| {
        for (k, parte) in salida.chunks_mut(trozo).enumerate() {
            s.spawn(move || {
                for (i, hueco) in parte.iter_mut().enumerate() {
                    let omega = k * trozo + i;
                    let e = generar(
                        inst.malla.len(),
                        inst.area,
                        inst.map_id,
                        omega,
                        entrada.semilla,
                        entrada.mc,
                    );
                    let p = ParamsTabu {
                        semilla: entrada.semilla.wrapping_add(omega as u64),
                        ..*params
                    };
                    *hueco = Some(resolver_escenario(
                        inst,
                        entrada.base,
                        &entrada.q,
                        &entrada.d,
                        &e,
                        &p,
                    ));
                }
            });
        }
    });
    let mut filas = Vec::with_capacity(n_omega);
    let mut phi0 = Vec::new();
    for (omega, h) in salida.into_iter().enumerate() {
        let (fila, phi) = h.expect("cada escenario se resuelve");
        if omega == 0 {
            phi0 = phi;
        }
        filas.push(fila);
    }
    let e0 = generar(
        inst.malla.len(),
        inst.area,
        inst.map_id,
        0,
        entrada.semilla,
        entrada.mc,
    );
    (filas, phi0, e0)
}

/// φ de cada posición de `paso1_patron_<inst>.csv` (especie −1 = vacía → φ = 0).
pub fn leer_patron(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut phi = Vec::new();
    for registro in lector.records() {
        let especie: i32 = registro?[5].parse()?;
        phi.push((especie + 1) as u8);
    }
    Ok(phi)
}

/// q de la fila (instancia, ε, π) de `paso2_compra.csv`.
pub fn leer_compra(
    path: &Path,
    instancia: &str,
    epsilon: f64,
    pi_factor: f64,
) -> Result<[u32; N_ESPECIES], Box<dyn Error>> {
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let enc = lector.headers()?.clone();
    let col = |nombre: &str| {
        enc.iter()
            .position(|h| h == nombre)
            .ok_or_else(|| format!("falta la columna {nombre}"))
    };
    let (ci, ce, cp) = (col("instancia")?, col("epsilon")?, col("pi_factor")?);
    let cq: Vec<usize> = (0..N_ESPECIES)
        .map(|j| col(&format!("q_{j}")))
        .collect::<Result<_, _>>()?;
    for registro in lector.records() {
        let r = registro?;
        if &r[ci] == instancia
            && (r[ce].parse::<f64>()? - epsilon).abs() < 1e-9
            && (r[cp].parse::<f64>()? - pi_factor).abs() < 1e-9
        {
            let mut q = [0u32; N_ESPECIES];
            for j in 0..N_ESPECIES {
                // Python puede escribir los enteros como 12.0.
                q[j] = r[cq[j]].parse::<f64>()?.round() as u32;
            }
            return Ok(q);
        }
    }
    Err(format!(
        "{}: sin fila para {instancia} (e={epsilon}, pi={pi_factor})",
        path.display()
    )
    .into())
}
