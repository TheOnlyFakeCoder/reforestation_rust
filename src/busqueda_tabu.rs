//! Búsqueda tabú sobre la malla (reemplaza al `tabu.rs` legacy en el modelo del Entregable 2).

use crate::competencia::TablaCompetencia;
use crate::datos::N_ESPECIES;
use crate::objetivo::{delta_intercambio, delta_reasignacion, z_total, Grafo};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Debug, PartialEq)]
pub enum Restriccion {
    Banda {
        l: Vec<usize>,
        u: Vec<usize>,
    },
    ComposicionFija,
    /// Paso 3: reasignación a → b (a, b ≠ vacía) con conteo_b + 1 ≤ n_b sobre las posiciones
    /// libres; los intercambios entre libres (también con vacías) dejan el total M fijo.
    Cuotas {
        n: Vec<usize>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamsTabu {
    pub candidatos: usize,
    pub tenure: usize,
    pub max_iter: usize,
    pub max_sin_mejora: usize,
    pub prob_intercambio: f64,
    pub semilla: u64,
}

pub struct Problema<'a> {
    pub grafo: &'a Grafo,
    pub tabla: &'a TablaCompetencia,
    pub fija: Vec<bool>,
    pub gamma: f64,
    pub theta: [f64; N_ESPECIES],
    pub restriccion: Restriccion,
}

#[derive(Clone, Debug)]
pub struct Resultado {
    pub phi: Vec<u8>,
    pub z: f64,
    pub objetivo: f64,
    pub iteraciones: usize,
    /// Objetivo del arreglo actual después de cada iteración.
    pub traza: Vec<f64>,
    /// Mejor objetivo visto hasta cada iteración (curva de convergencia).
    pub curva: Vec<f64>,
    pub tiempo_s: f64,
}

impl Problema<'_> {
    /// γ·Z(φ) + Σ_j θ_j·D_j; las vacías no cuentan.
    pub fn objetivo(&self, phi: &[u8]) -> f64 {
        let mortalidad: f64 = phi
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| self.theta[c as usize - 1])
            .sum();
        self.gamma * z_total(phi, self.grafo, self.tabla) + mortalidad
    }
}

pub fn buscar(problema: &Problema, inicial: Vec<u8>, params: &ParamsTabu) -> Resultado {
    let inicio = std::time::Instant::now();
    let mut phi = inicial;
    // Las posiciones fijas (existentes) nunca se mueven, pero siguen contando como vecinas.
    let libres: Vec<usize> = (0..phi.len()).filter(|&u| !problema.fija[u]).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(params.semilla);
    let mut iteraciones = 0;
    let mut sin_mejora = 0;
    let mut actual = problema.objetivo(&phi);
    let mut traza = Vec::new();
    let mut curva = Vec::new();
    let mut conteos = [0usize; N_ESPECIES + 1];
    // Con cuotas sólo cuentan las posiciones libres (las existentes no consumen cuota).
    let solo_libres = matches!(problema.restriccion, Restriccion::Cuotas { .. });
    for (u, &c) in phi.iter().enumerate() {
        if !(solo_libres && problema.fija[u]) {
            conteos[c as usize] += 1;
        }
    }
    let mut mejor_objetivo = actual;
    let mut mejor_phi = phi.clone();
    // tabu_hasta[u][s]: iteración hasta la que u no puede volver a tener la especie s.
    let mut tabu_hasta = vec![[0usize; N_ESPECIES + 1]; phi.len()];
    while iteraciones < params.max_iter && sin_mejora < params.max_sin_mejora && libres.len() > 1 {
        iteraciones += 1;
        // Mejor de `candidatos` movimientos al azar; se aplica aunque empeore.
        let mut elegido: Option<(f64, Movimiento)> = None;
        for _ in 0..params.candidatos {
            let u = libres[rng.gen_range(0..libres.len())];
            let candidato = match &problema.restriccion {
                // Reasignación a → b dentro de [L, U]; la vacía no se reasigna.
                Restriccion::Banda { l, u: tope }
                    if rng.gen::<f64>() >= params.prob_intercambio =>
                {
                    let (a, b) = (phi[u], rng.gen_range(1..=N_ESPECIES as u8));
                    if a == 0
                        || a == b
                        || conteos[a as usize] - 1 < l[a as usize - 1]
                        || conteos[b as usize] + 1 > tope[b as usize - 1]
                    {
                        continue;
                    }
                    let delta = problema.gamma
                        * delta_reasignacion(&phi, u, b, problema.grafo, problema.tabla)
                        + problema.theta[b as usize - 1]
                        - problema.theta[a as usize - 1];
                    let es_tabu = tabu_hasta[u][b as usize] > iteraciones;
                    if !admisible(es_tabu, actual, delta, mejor_objetivo) {
                        continue;
                    }
                    (delta, Movimiento::Reasignar { u, b })
                }
                // Reasignación a → b con tope n_b; el total de plantas no cambia de especie-vacía.
                Restriccion::Cuotas { n: cuotas }
                    if rng.gen::<f64>() >= params.prob_intercambio =>
                {
                    let (a, b) = (phi[u], rng.gen_range(1..=N_ESPECIES as u8));
                    if a == 0 || a == b || conteos[b as usize] + 1 > cuotas[b as usize - 1] {
                        continue;
                    }
                    let delta = problema.gamma
                        * delta_reasignacion(&phi, u, b, problema.grafo, problema.tabla)
                        + problema.theta[b as usize - 1]
                        - problema.theta[a as usize - 1];
                    let es_tabu = tabu_hasta[u][b as usize] > iteraciones;
                    if !admisible(es_tabu, actual, delta, mejor_objetivo) {
                        continue;
                    }
                    (delta, Movimiento::Reasignar { u, b })
                }
                _ => {
                    let v = libres[rng.gen_range(0..libres.len())];
                    if phi[u] == phi[v] {
                        continue;
                    }
                    let delta = problema.gamma
                        * delta_intercambio(&phi, u, v, problema.grafo, problema.tabla);
                    let es_tabu = tabu_hasta[u][phi[v] as usize] > iteraciones
                        || tabu_hasta[v][phi[u] as usize] > iteraciones;
                    if !admisible(es_tabu, actual, delta, mejor_objetivo) {
                        continue;
                    }
                    (delta, Movimiento::Intercambiar { u, v })
                }
            };
            if elegido.as_ref().is_none_or(|(d, _)| candidato.0 < *d) {
                elegido = Some(candidato);
            }
        }
        if let Some((delta, movimiento)) = elegido {
            let hasta = iteraciones + params.tenure;
            match movimiento {
                Movimiento::Reasignar { u, b } => {
                    tabu_hasta[u][phi[u] as usize] = hasta;
                    conteos[phi[u] as usize] -= 1;
                    conteos[b as usize] += 1;
                    phi[u] = b;
                }
                Movimiento::Intercambiar { u, v } => {
                    tabu_hasta[u][phi[u] as usize] = hasta;
                    tabu_hasta[v][phi[v] as usize] = hasta;
                    phi.swap(u, v);
                }
            }
            actual += delta;
        }
        traza.push(actual);
        if actual < mejor_objetivo - 1e-12 {
            mejor_objetivo = actual;
            mejor_phi.copy_from_slice(&phi);
            sin_mejora = 0;
        } else {
            sin_mejora += 1;
        }
        curva.push(mejor_objetivo);
    }
    Resultado {
        z: z_total(&mejor_phi, problema.grafo, problema.tabla),
        objetivo: problema.objetivo(&mejor_phi),
        phi: mejor_phi,
        iteraciones,
        traza,
        curva,
        tiempo_s: inicio.elapsed().as_secs_f64(),
    }
}

/// Criterio de aspiración: un movimiento tabú sólo se admite si mejora la mejor solución.
fn admisible(es_tabu: bool, actual: f64, delta: f64, mejor: f64) -> bool {
    !es_tabu || actual + delta < mejor - 1e-12
}

#[derive(Clone, Copy, Debug)]
enum Movimiento {
    Reasignar { u: usize, b: u8 },
    Intercambiar { u: usize, v: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspiracion_admite_un_movimiento_tabu_que_supera_al_mejor() {
        // actual 10, mejor 9: un tabú con delta −2 llega a 8 < 9 y se admite.
        assert!(admisible(true, 10.0, -2.0, 9.0));
        // Un tabú que no supera al mejor (llega a 9.5) se descarta.
        assert!(!admisible(true, 10.0, -0.5, 9.0));
        // Un movimiento no tabú siempre es admisible, aunque empeore.
        assert!(admisible(false, 10.0, 3.0, 9.0));
    }
}
