//! Competencia total Z de un arreglo φ y sus cambios por movimiento.
//!
//! φ(u) usa la codificación de `competencia`: 0 = vacía, especie j ↔ j + 1.

use crate::competencia::TablaCompetencia;
use crate::malla::{Vecindad, Vecino};

/// Vecindad en formato compacto (CSR): los vecinos de u son `vecinos[inicio[u]..inicio[u + 1]]`.
#[derive(Clone, Debug)]
pub struct Grafo {
    inicio: Vec<u32>,
    vecinos: Vec<Vecino>,
}

impl Grafo {
    pub fn desde(vecindad: &Vecindad) -> Grafo {
        let mut inicio = Vec::with_capacity(vecindad.vecinos.len() + 1);
        let mut vecinos = Vec::new();
        for lista in &vecindad.vecinos {
            inicio.push(vecinos.len() as u32);
            vecinos.extend_from_slice(lista);
        }
        inicio.push(vecinos.len() as u32);
        Grafo { inicio, vecinos }
    }

    #[inline]
    pub fn vecinos(&self, u: usize) -> &[Vecino] {
        &self.vecinos[self.inicio[u] as usize..self.inicio[u + 1] as usize]
    }
}

/// ΔZ al cambiar φ(u) = a por b: Σ_{z∈N(u)} [w_b,φz·h − w_a,φz·h].
pub fn delta_reasignacion(
    phi: &[u8],
    u: usize,
    b: u8,
    grafo: &Grafo,
    tabla: &TablaCompetencia,
) -> f64 {
    let a = phi[u];
    grafo
        .vecinos(u)
        .iter()
        .map(|z| {
            let c = phi[z.pos as usize];
            tabla.peso(z.anillo, b, c) - tabla.peso(z.anillo, a, c)
        })
        .sum()
}

/// ΔZ al intercambiar φ(u) = a y φ(v) = b:
/// Σ_{z∈N(u)∖{v}} [w_b,φz·h − w_a,φz·h] + Σ_{z∈N(v)∖{u}} [w_a,φz·h − w_b,φz·h].
/// Si u y v son vecinos, el par (u, v) no cambia porque W es simétrica.
pub fn delta_intercambio(
    phi: &[u8],
    u: usize,
    v: usize,
    grafo: &Grafo,
    tabla: &TablaCompetencia,
) -> f64 {
    let (a, b) = (phi[u], phi[v]);
    let lado = |desde: usize, otro: usize, sale: u8, entra: u8| -> f64 {
        grafo
            .vecinos(desde)
            .iter()
            .filter(|z| z.pos as usize != otro)
            .map(|z| {
                let c = phi[z.pos as usize];
                tabla.peso(z.anillo, entra, c) - tabla.peso(z.anillo, sale, c)
            })
            .sum()
    };
    lado(u, v, a, b) + lado(v, u, b, a)
}

/// Z = Σ_{pares {u, v}} w·h, cada par contado una vez.
pub fn z_total(phi: &[u8], grafo: &Grafo, tabla: &TablaCompetencia) -> f64 {
    let mut z = 0.0;
    for (u, &a) in phi.iter().enumerate() {
        for vecino in grafo.vecinos(u) {
            let v = vecino.pos as usize;
            if v > u {
                z += tabla.peso(vecino.anillo, a, phi[v]);
            }
        }
    }
    z
}
