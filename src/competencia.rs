//! Competencia entre especies: matriz W (similitud de Gower), factor de alcance h_jk(δ) y la
//! tabla w·h por anillo que usa la búsqueda tabú.
//!
//! Codificación de φ: 0 = posición vacía, especie j ↔ j + 1.

use crate::datos::Especie;

pub const VACIA: u8 = 0;

/// Similitud de Gower S_jk con 7 rasgos de peso 1/7: familia, género, forma, CAM y fija N
/// (1 si coinciden), raíz 1 − |r_j − r_k|/2 y altura 1 − |ln h_j − ln h_k| / ln(h_max/h_min).
pub fn similitud_gower(a: &Especie, b: &Especie, ln_rango_altura: f64) -> f64 {
    let igual = |x: bool| if x { 1.0 } else { 0.0 };
    let categoricos = igual(a.familia == b.familia)
        + igual(a.genero == b.genero)
        + igual(a.forma == b.forma)
        + igual(a.cam == b.cam)
        + igual(a.fija_n == b.fija_n);
    let raiz = 1.0 - (a.raiz as f64 - b.raiz as f64).abs() / 2.0;
    let altura = 1.0 - (a.altura.ln() - b.altura.ln()).abs() / ln_rango_altura;
    (categoricos + raiz + altura) / 7.0
}

/// w_jj = 1; w_jk = κ·S_jk para j ≠ k.
pub fn matriz_w(especies: &[Especie], kappa: f64) -> Vec<Vec<f64>> {
    let h_max = especies.iter().map(|e| e.altura).fold(f64::MIN, f64::max);
    let h_min = especies.iter().map(|e| e.altura).fold(f64::MAX, f64::min);
    let ln_rango = (h_max / h_min).ln();
    especies
        .iter()
        .enumerate()
        .map(|(j, a)| {
            especies
                .iter()
                .enumerate()
                .map(|(k, b)| {
                    if j == k {
                        1.0
                    } else {
                        kappa * similitud_gower(a, b, ln_rango)
                    }
                })
                .collect()
        })
        .collect()
}

/// R_j = (R_j/d)·d·factor.
pub fn radios(especies: &[Especie], d: f64, factor: f64) -> Vec<f64> {
    especies.iter().map(|e| e.radio_d * d * factor).collect()
}

/// Alcance de la vecindad: max(max_j R_j, d), porque los vecinos inmediatos siempre compiten.
pub fn radio_max(radios: &[f64], d: f64) -> f64 {
    radios.iter().copied().fold(d, f64::max)
}

/// h_jk(δ): 1 si δ ≤ d; decae linealmente hasta 0 en R_jk; 0 si δ ≥ R_jk.
pub fn alcance(delta: f64, d: f64, r_jk: f64, tol: f64) -> f64 {
    if delta <= d + tol {
        1.0
    } else if delta >= r_jk - tol {
        0.0
    } else {
        1.0 - (delta - d) / (r_jk - d)
    }
}

/// Peso w_ab·h_ab(δ_anillo) precalculado para cada anillo y cada par (a, b) en codificación φ
/// (0 = vacía, que pesa 0 con todas).
#[derive(Clone, Debug)]
pub struct TablaCompetencia {
    n: usize,
    n_anillos: usize,
    wh: Vec<f64>,
}

impl TablaCompetencia {
    pub fn nueva(
        w: &[Vec<f64>],
        radios: &[f64],
        anillos: &[f64],
        d: f64,
        tol: f64,
    ) -> TablaCompetencia {
        let n = w.len() + 1;
        let mut wh = vec![0.0; anillos.len() * n * n];
        for (anillo, &delta) in anillos.iter().enumerate() {
            for j in 0..w.len() {
                for k in 0..w.len() {
                    let r_jk = radios[j].max(radios[k]);
                    wh[(anillo * n + j + 1) * n + k + 1] = w[j][k] * alcance(delta, d, r_jk, tol);
                }
            }
        }
        TablaCompetencia {
            n,
            n_anillos: anillos.len(),
            wh,
        }
    }

    pub fn n_anillos(&self) -> usize {
        self.n_anillos
    }

    #[inline]
    pub fn peso(&self, anillo: u8, a: u8, b: u8) -> f64 {
        self.wh[(anillo as usize * self.n + a as usize) * self.n + b as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datos::Datos;
    const D: f64 = 4.189171180993856;

    fn w() -> Vec<Vec<f64>> {
        matriz_w(&Datos::base().especies, 0.6)
    }

    fn r2(x: f64) -> f64 {
        (x * 100.0).round() / 100.0
    }

    #[test]
    fn w_tiene_diagonal_uno_y_es_simetrica() {
        let w = w();
        for j in 0..10 {
            assert_eq!(w[j][j], 1.0);
            for k in 0..10 {
                assert!((w[j][k] - w[k][j]).abs() < 1e-15);
            }
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
            for j in a.clone() {
                for k in b.clone() {
                    if j != k {
                        v.push((j, k));
                    }
                }
            }
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
        for anillo in 0..3 {
            for a in 0..=10 {
                assert_eq!(tabla.peso(anillo, VACIA, a), 0.0);
                assert_eq!(tabla.peso(anillo, a, VACIA), 0.0);
            }
        }
    }
}
