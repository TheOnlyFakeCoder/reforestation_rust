//! Conteos de transición entre especies de un patrón: por vecindad inmediata (pares a distancia d)
//! y por el orden de siembra en serpentina (sólo plantas nuevas).

use std::error::Error;
use std::path::Path;

use crate::datos::N_ESPECIES;
use crate::malla::Malla;
use crate::objetivo::Grafo;

type Matriz = [[u64; N_ESPECIES]; N_ESPECIES];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Conteos {
    /// Pares a distancia d, ambas plantadas (nuevas o existentes), en los dos sentidos.
    pub vecindad: Matriz,
    /// Pares consecutivos de plantas nuevas en el orden de siembra.
    pub orden: Matriz,
    /// Plantas por especie (nuevas + existentes).
    pub plantas: [u64; N_ESPECIES],
}

impl Conteos {
    pub fn sumar(&mut self, otro: &Conteos) {
        for j in 0..N_ESPECIES {
            self.plantas[j] += otro.plantas[j];
            for k in 0..N_ESPECIES {
                self.vecindad[j][k] += otro.vecindad[j][k];
                self.orden[j][k] += otro.orden[j][k];
            }
        }
    }
}

/// Posiciones de las plantas nuevas (φ > 0 y no existente) en orden de siembra: filas crecientes;
/// dentro de la fila, `col` creciente si la fila es par y decreciente si es impar.
pub fn orden_serpentina(malla: &Malla, phi: &[u8], existente: &[bool]) -> Vec<usize> {
    let mut orden: Vec<usize> = (0..phi.len())
        .filter(|&u| phi[u] > 0 && !existente[u])
        .collect();
    orden.sort_by_key(|&u| {
        let p = &malla.posiciones[u];
        let col = if p.fila % 2 == 0 {
            p.col as i64
        } else {
            -(p.col as i64)
        };
        (p.fila, col)
    });
    orden
}

pub fn conteos(malla: &Malla, grafo: &Grafo, phi: &[u8], existente: &[bool]) -> Conteos {
    let mut c = Conteos::default();
    for u in 0..phi.len() {
        if phi[u] == 0 {
            continue;
        }
        let a = phi[u] as usize - 1;
        c.plantas[a] += 1;
        for v in grafo.vecinos(u) {
            let b = phi[v.pos as usize];
            if v.anillo == 0 && b > 0 {
                c.vecindad[a][b as usize - 1] += 1;
            }
        }
    }
    let orden = orden_serpentina(malla, phi, existente);
    for par in orden.windows(2) {
        c.orden[phi[par[0]] as usize - 1][phi[par[1]] as usize - 1] += 1;
    }
    c
}

/// CSV `tipo, de, a, conteo, n, prob` (200 filas): n = Σ conteos de la fila, prob = conteo/n o 0.
pub fn escribir_transicion(path: &Path, c: &Conteos) -> Result<(), Box<dyn Error>> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(["tipo", "de", "a", "conteo", "n", "prob"])?;
    for (tipo, m) in [("vecindad", &c.vecindad), ("orden", &c.orden)] {
        for j in 0..N_ESPECIES {
            let n: u64 = m[j].iter().sum();
            for k in 0..N_ESPECIES {
                let prob = if n > 0 {
                    m[j][k] as f64 / n as f64
                } else {
                    0.0
                };
                w.write_record([
                    tipo.to_string(),
                    j.to_string(),
                    k.to_string(),
                    m[j][k].to_string(),
                    n.to_string(),
                    prob.to_string(),
                ])?;
            }
        }
    }
    w.flush()?;
    Ok(())
}

/// CSV `especie, plantas, fraccion`.
pub fn escribir_plantas(path: &Path, c: &Conteos) -> Result<(), Box<dyn Error>> {
    let total: u64 = c.plantas.iter().sum();
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(["especie", "plantas", "fraccion"])?;
    for j in 0..N_ESPECIES {
        let f = if total > 0 {
            c.plantas[j] as f64 / total as f64
        } else {
            0.0
        };
        w.write_record([j.to_string(), c.plantas[j].to_string(), f.to_string()])?;
    }
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::malla::Disposicion;

    // Malla::rectangular(3, 2, Completa, d = 1.0): fila 0 = pos 0,1,2 (x = .5, 1.5, 2.5);
    // fila 1 = pos 3,4,5 (x = 1, 2, 3). Adyacencias a distancia d: horizontales 0-1, 1-2, 3-4,
    // 4-5 y diagonales 0-3, 1-3, 1-4, 2-4, 2-5.
    fn caso() -> (Malla, Grafo, Vec<u8>, Vec<bool>) {
        let malla = Malla::rectangular(3, 2, Disposicion::Completa, 1.0);
        let grafo = Grafo::desde(&malla.vecindad(1.0, 1e-9));
        let phi = vec![1, 2, 1, 0, 3, 2];
        let existente = vec![false, false, false, false, true, false];
        (malla, grafo, phi, existente)
    }

    #[test]
    fn orden_salta_vacias_y_existentes() {
        let (m, _, phi, ex) = caso();
        assert_eq!(orden_serpentina(&m, &phi, &ex), vec![0, 1, 2, 5]);
    }

    #[test]
    fn conteos_del_caso_fijo() {
        let (m, g, phi, ex) = caso();
        let c = conteos(&m, &g, &phi, &ex);
        let mut orden = [[0u64; N_ESPECIES]; N_ESPECIES];
        orden[0][1] = 2;
        orden[1][0] = 1;
        assert_eq!(c.orden, orden);
        // Aristas entre plantadas (sin la vacía 3): 0-1 (0,1), 1-2 (1,0), 2-5 (0,1), 1-4 (1,2),
        // 2-4 (0,2), 4-5 (2,1); cada una en los dos sentidos.
        let mut vec = [[0u64; N_ESPECIES]; N_ESPECIES];
        vec[0][1] = 3;
        vec[1][0] = 3;
        vec[1][2] = 2;
        vec[2][1] = 2;
        vec[0][2] = 1;
        vec[2][0] = 1;
        assert_eq!(c.vecindad, vec);
        let mut plantas = [0u64; N_ESPECIES];
        plantas[0] = 2;
        plantas[1] = 2;
        plantas[2] = 1;
        assert_eq!(c.plantas, plantas);
    }

    #[test]
    fn sumar_acumula() {
        let (m, g, phi, ex) = caso();
        let c = conteos(&m, &g, &phi, &ex);
        let mut s = Conteos::default();
        s.sumar(&c);
        s.sumar(&c);
        assert_eq!(s.vecindad[0][1], 6);
        assert_eq!(s.orden[0][1], 4);
        assert_eq!(s.plantas[0], 4);
    }

    #[test]
    fn escribe_200_filas_con_prob_que_suma_1_o_0() {
        let (m, g, phi, ex) = caso();
        let c = conteos(&m, &g, &phi, &ex);
        let dir = std::env::temp_dir().join(format!("trans_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("t.csv");
        escribir_transicion(&p, &c).unwrap();
        let mut r = csv::Reader::from_path(&p).unwrap();
        let mut suma = std::collections::HashMap::new();
        let mut filas = 0;
        for reg in r.records() {
            let reg = reg.unwrap();
            filas += 1;
            let clave = (reg[0].to_string(), reg[1].to_string());
            *suma.entry(clave).or_insert(0.0) += reg[5].parse::<f64>().unwrap();
        }
        assert_eq!(filas, 200);
        for s in suma.values() {
            assert!((s - 1.0).abs() < 1e-9 || s.abs() < 1e-12, "{s}");
        }
        let q = dir.join("p.csv");
        escribir_plantas(&q, &c).unwrap();
        assert_eq!(
            csv::Reader::from_path(&q).unwrap().records().count(),
            N_ESPECIES
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
