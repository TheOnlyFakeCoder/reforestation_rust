//! Coloración DSATUR por cuotas: línea base constructiva para comparar con el tabú.

use crate::competencia::TablaCompetencia;
use crate::datos::N_ESPECIES;
use crate::objetivo::Grafo;

/// Reparte las cuotas entre las posiciones libres (`fijas[u] == 0`) con DSATUR.
///
/// Devuelve φ completo (0 = vacía, especie j ↔ j+1) con las fijas intactas. Determinista:
/// 1. Se elige la libre sin asignar con mayor saturación (especies distintas entre sus
///    vecinos del anillo 0 ya plantados); empate: más vecinos; empate: menor índice.
/// 2. Se le asigna la especie con cuota restante que minimiza la competencia con los vecinos
///    plantados; empate: mayor cuota restante; empate: menor j.
/// 3. Al agotarse las cuotas las libres restantes quedan vacías.
pub fn dsatur_cuotas(
    grafo: &Grafo,
    tabla: &TablaCompetencia,
    cuotas: &[usize; N_ESPECIES],
    fijas: &[u8],
) -> Vec<u8> {
    let n = fijas.len();
    let mut phi = fijas.to_vec();
    let mut restante = *cuotas;
    // Máscara de especies presentes entre los vecinos inmediatos plantados.
    let mut vistas = vec![0u16; n];
    for u in 0..n {
        if phi[u] > 0 {
            marcar(grafo, &mut vistas, u, phi[u]);
        }
    }
    let grado: Vec<usize> = (0..n).map(|u| grafo.vecinos(u).len()).collect();
    let mut sin_asignar: Vec<bool> = fijas.iter().map(|&f| f == 0).collect();
    while restante.iter().any(|&c| c > 0) {
        let mut mejor: Option<usize> = None;
        for u in 0..n {
            if !sin_asignar[u] {
                continue;
            }
            mejor = match mejor {
                None => Some(u),
                Some(b) => {
                    let (su, sb) = (vistas[u].count_ones(), vistas[b].count_ones());
                    if su > sb || (su == sb && grado[u] > grado[b]) {
                        Some(u)
                    } else {
                        Some(b)
                    }
                }
            };
        }
        let Some(u) = mejor else { break };
        let mut elegida: Option<(usize, f64)> = None;
        for j in 0..N_ESPECIES {
            if restante[j] == 0 {
                continue;
            }
            let costo: f64 = grafo
                .vecinos(u)
                .iter()
                .filter(|v| phi[v.pos as usize] > 0)
                .map(|v| tabla.peso(v.anillo, j as u8 + 1, phi[v.pos as usize]))
                .sum();
            let mejor_j = match elegida {
                None => true,
                Some((k, c)) => costo < c || (costo == c && restante[j] > restante[k]),
            };
            if mejor_j {
                elegida = Some((j, costo));
            }
        }
        let (j, _) = elegida.expect("hay cuota restante");
        phi[u] = j as u8 + 1;
        restante[j] -= 1;
        sin_asignar[u] = false;
        marcar(grafo, &mut vistas, u, phi[u]);
    }
    phi
}

fn marcar(grafo: &Grafo, vistas: &mut [u16], u: usize, especie: u8) {
    for v in grafo.vecinos(u) {
        if v.anillo == 0 {
            vistas[v.pos as usize] |= 1 << (especie - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::malla::{Disposicion, Malla};

    const D: f64 = 1.0;

    /// Malla rectangular con vecindad a distancia d; sólo compiten las plantas de la misma especie.
    fn caso(columnas: usize, filas: usize) -> (Grafo, TablaCompetencia, usize) {
        let malla = Malla::rectangular(columnas, filas, Disposicion::Completa, D);
        let grafo = Grafo::desde(&malla.vecindad(D, 1e-6));
        let w: Vec<Vec<f64>> = (0..N_ESPECIES)
            .map(|j| (0..N_ESPECIES).map(|k| (j == k) as u8 as f64).collect())
            .collect();
        let tabla = TablaCompetencia::nueva(&w, &[D; N_ESPECIES], &[D], D, 1e-6);
        (grafo, tabla, malla.len())
    }

    fn cuenta(phi: &[u8]) -> [usize; N_ESPECIES] {
        let mut c = [0; N_ESPECIES];
        for &f in phi {
            if f > 0 {
                c[f as usize - 1] += 1;
            }
        }
        c
    }

    #[test]
    fn dsatur_respeta_cuotas() {
        let (g, t, n) = caso(5, 5);
        let cuotas = [10, 8, 7, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(cuotas.iter().sum::<usize>(), n);
        let phi = dsatur_cuotas(&g, &t, &cuotas, &vec![0; n]);
        assert_eq!(cuenta(&phi), cuotas);
        assert!(phi.iter().all(|&f| f > 0));
    }

    #[test]
    fn dsatur_respeta_cuotas_con_huecos() {
        let (g, t, n) = caso(5, 5);
        let cuotas = [8, 8, 6, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(cuotas.iter().sum::<usize>(), n - 3);
        let phi = dsatur_cuotas(&g, &t, &cuotas, &vec![0; n]);
        assert_eq!(cuenta(&phi), cuotas);
        assert_eq!(phi.iter().filter(|&&f| f == 0).count(), 3);
    }

    #[test]
    fn dsatur_no_toca_fijas() {
        let (g, t, n) = caso(5, 5);
        let mut fijas = vec![0u8; n];
        fijas[0] = 4;
        fijas[7] = 2;
        fijas[24] = 9;
        let cuotas = [10, 5, 5, 0, 0, 0, 0, 0, 0, 0];
        let phi = dsatur_cuotas(&g, &t, &cuotas, &fijas);
        for u in 0..n {
            if fijas[u] > 0 {
                assert_eq!(phi[u], fijas[u]);
            }
        }
        let mut esperado = cuotas;
        esperado[3] += 1;
        esperado[1] += 1;
        esperado[8] += 1;
        assert_eq!(cuenta(&phi), esperado);
    }

    #[test]
    fn dsatur_determinista() {
        let (g, t, n) = caso(6, 6);
        let cuotas = [10, 10, 10, 3, 0, 0, 0, 0, 0, 0];
        let a = dsatur_cuotas(&g, &t, &cuotas, &vec![0; n]);
        let b = dsatur_cuotas(&g, &t, &cuotas, &vec![0; n]);
        assert_eq!(a, b);
    }

    #[test]
    fn dsatur_evita_vecinos_misma_especie_con_tres_especies() {
        let (g, t, n) = caso(6, 6);
        assert_eq!(n, 36);
        let cuotas = [12, 12, 12, 0, 0, 0, 0, 0, 0, 0];
        let phi = dsatur_cuotas(&g, &t, &cuotas, &vec![0; n]);
        let mut pares = 0;
        for u in 0..n {
            for v in g.vecinos(u) {
                if v.anillo == 0 && phi[u] > 0 && phi[u] == phi[v.pos as usize] {
                    pares += 1;
                }
            }
        }
        assert_eq!(
            pares, 0,
            "pares inmediatos de la misma especie (ambos sentidos)"
        );
    }
}
