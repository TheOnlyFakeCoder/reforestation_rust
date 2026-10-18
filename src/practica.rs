//! Práctica actual: la secuencia fija de 21 plantas de la socia en serpentina, comparada con los
//! patrones de la búsqueda tabú por competencia Z y su desglose por nivel de competencia.
//!
//! Niveles de un par de especies (j, k): 1 misma especie; 2 mismo género; 4 Yucca (9)–Prosopis (8);
//! 5 Prosopis (8) con una de las especies 0..7; 3 si 0.2 ≤ w_jk ≤ 0.5; "otro" el resto. Los
//! niveles 4 y 5 se revisan antes que el 3 para que las clases sean disjuntas.

use crate::competencia::TablaCompetencia;
use crate::datos::Especie;
use crate::malla::Malla;
use crate::objetivo::Grafo;

const YUCCA: usize = 9;
const PROSOPIS: usize = 8;

/// Índice en `Desglose::niveles`: 0..=4 niveles 1–5 y 5 = "otro".
pub const OTRO: usize = 5;

/// Nivel (0-based: 0 = nivel 1, …, 4 = nivel 5, 5 = otro) del par de especies j, k (0-based).
pub fn nivel(j: usize, k: usize, especies: &[Especie], w: &[Vec<f64>]) -> usize {
    if j == k {
        0
    } else if especies[j].genero == especies[k].genero {
        1
    } else if (j == YUCCA && k == PROSOPIS) || (j == PROSOPIS && k == YUCCA) {
        3
    } else if (j == PROSOPIS && k < PROSOPIS) || (k == PROSOPIS && j < PROSOPIS) {
        4
    } else if (0.2..=0.5).contains(&w[j][k]) {
        2
    } else {
        OTRO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Desglose {
    pub plantas: usize,
    pub z: f64,
    /// Z por nivel 1–5 y "otro".
    pub niveles: [f64; 6],
    /// Pares no ordenados de la misma especie a la distancia d (anillo 0).
    pub pares_inmediatos: usize,
}

/// Z de φ (codificación 0 = vacía) y su desglose por nivel; cada par se cuenta una vez.
pub fn desglose(
    phi: &[u8],
    grafo: &Grafo,
    tabla: &TablaCompetencia,
    especies: &[Especie],
    w: &[Vec<f64>],
) -> Desglose {
    let mut d = Desglose {
        plantas: phi.iter().filter(|&&c| c > 0).count(),
        z: 0.0,
        niveles: [0.0; 6],
        pares_inmediatos: 0,
    };
    for (u, &a) in phi.iter().enumerate() {
        if a == 0 {
            continue;
        }
        for vecino in grafo.vecinos(u) {
            let v = vecino.pos as usize;
            let b = phi[v];
            if v <= u || b == 0 {
                continue;
            }
            let peso = tabla.peso(vecino.anillo, a, b);
            d.z += peso;
            d.niveles[nivel(a as usize - 1, b as usize - 1, especies, w)] += peso;
            if vecino.anillo == 0 && a == b {
                d.pares_inmediatos += 1;
            }
        }
    }
    d
}

/// Variante iii: parte de las existentes fijas y recorre la serpentina (filas pares de izquierda
/// a derecha, impares al revés); la secuencia (cíclica) se asigna sólo a posiciones libres,
/// saltando las ocupadas, hasta plantar `m` nuevas. El resto queda vacío.
pub fn serpentina_con_existentes(
    malla: &Malla,
    existentes: &[(usize, u8)],
    secuencia: &[usize],
    m: usize,
) -> Vec<u8> {
    let mut phi = vec![0u8; malla.len()];
    for &(u, j) in existentes {
        phi[u] = j + 1;
    }
    let ocupada: Vec<bool> = phi.iter().map(|&c| c > 0).collect();
    let recorrido = (0..malla.filas()).flat_map(|fila| {
        let posiciones: Vec<usize> = malla.fila(fila).collect();
        if fila % 2 == 0 {
            posiciones
        } else {
            posiciones.into_iter().rev().collect()
        }
    });
    let libres = recorrido.filter(|&u| !ocupada[u]).take(m);
    for (k, u) in libres.enumerate() {
        phi[u] = secuencia[k % secuencia.len()] as u8 + 1;
    }
    phi
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competencia::matriz_w;
    use crate::datos::{Datos, N_ESPECIES};
    use crate::tabu::DEFAULT_ROW_SEQUENCE;

    #[test]
    fn niveles_disjuntos_y_prioridad() {
        let datos = Datos::base();
        let w = matriz_w(&datos.especies, 0.6);
        assert_eq!(nivel(3, 3, &datos.especies, &w), 0);
        assert_eq!(nivel(YUCCA, PROSOPIS, &datos.especies, &w), 3);
        assert_eq!(nivel(PROSOPIS, YUCCA, &datos.especies, &w), 3);
        for k in 0..PROSOPIS {
            let n = nivel(PROSOPIS, k, &datos.especies, &w);
            assert!(n == 4 || n == 1, "Prosopis con {k} dio nivel {n}");
        }
        for j in 0..N_ESPECIES {
            for k in 0..N_ESPECIES {
                assert_eq!(
                    nivel(j, k, &datos.especies, &w),
                    nivel(k, j, &datos.especies, &w)
                );
            }
        }
    }

    fn montaje() -> (Datos, Vec<Vec<f64>>, crate::paso1::Instancia) {
        let datos = Datos::base();
        let config = crate::config::Config::default();
        let w = matriz_w(&datos.especies, config.kappa);
        let inst = crate::paso1::Instancia::nueva("1ha", 1.0, &datos, &config, 1.0, false);
        (datos, w, inst)
    }

    #[test]
    fn suma_de_niveles_es_z() {
        let (datos, w, inst) = montaje();
        let phi = crate::paso1::patron_serpentina(&inst.malla, DEFAULT_ROW_SEQUENCE, inst.d_tot);
        let d = desglose(&phi, &inst.grafo, &inst.tabla, &datos.especies, &w);
        let z = crate::objetivo::z_total(&phi, &inst.grafo, &inst.tabla);
        assert!((d.z - z).abs() < 1e-9);
        assert!((d.niveles.iter().sum::<f64>() - z).abs() < 1e-9);
        assert_eq!(d.plantas, inst.d_tot);
    }

    #[test]
    fn pares_inmediatos_de_arreglo_uniforme() {
        let (datos, w, inst) = montaje();
        let phi = vec![1u8; inst.malla.len()];
        let d = desglose(&phi, &inst.grafo, &inst.tabla, &datos.especies, &w);
        let esperado: usize = (0..phi.len())
            .map(|u| {
                inst.grafo
                    .vecinos(u)
                    .iter()
                    .filter(|v| v.anillo == 0 && v.pos as usize > u)
                    .count()
            })
            .sum();
        assert_eq!(d.pares_inmediatos, esperado);
        assert!(esperado > 0);
        assert!((d.niveles[0] - d.z).abs() < 1e-9);
    }

    #[test]
    fn iii_no_toca_existentes_y_planta_m() {
        let (_, _, inst) = montaje();
        let existentes: Vec<(usize, u8)> = vec![(0, 5), (1, 2), (10, 7), (100, 0)];
        let phi = serpentina_con_existentes(&inst.malla, &existentes, DEFAULT_ROW_SEQUENCE, 50);
        for &(u, j) in &existentes {
            assert_eq!(phi[u], j + 1);
        }
        assert_eq!(
            phi.iter().filter(|&&c| c > 0).count(),
            existentes.len() + 50
        );
        // la secuencia se asigna en orden a las libres, saltando ocupadas: la primera libre del
        // recorrido (posición 2 de la fila 0) recibe el primer elemento
        assert_eq!(phi[2] as usize, DEFAULT_ROW_SEQUENCE[0] + 1);
        assert_eq!(phi[3] as usize, DEFAULT_ROW_SEQUENCE[1] + 1);
    }
}
