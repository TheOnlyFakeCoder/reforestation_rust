//! Malla tresbolillo (hexagonal) sobre un cuadrado, con coordenadas en metros.
//!
//! Los centros quedan a medio paso del borde: x = d/2 + c·d en filas pares y x = d + c·d en
//! impares; y = h/2 + f·h con h = d·√3/2. Así cada posición ocupa su celda hexagonal de área
//! (√3/2)·d² dentro del cuadrado y |V| ≈ 658·A con d = 4.19 m.

/// sen 60° = √3/2: separación entre filas en múltiplos de d.
pub const SENO_60: f64 = 0.866_025_403_784_438_6;

/// Holgura para incluir puntos que caen justo en el borde por redondeo.
const HOLGURA_BORDE: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Posicion {
    pub fila: usize,
    pub col: usize,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct Malla {
    pub d: f64,
    pub lado: f64,
    pub posiciones: Vec<Posicion>,
    /// Índice de la primera posición de cada fila; el último elemento es `len()`.
    inicio_fila: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vecino {
    pub pos: u32,
    pub anillo: u8,
}

/// Vecinos de cada posición agrupados por anillo (distancia distinta en la malla).
#[derive(Clone, Debug)]
pub struct Vecindad {
    /// Distancia de cada anillo (m), en orden creciente.
    pub anillos: Vec<f64>,
    pub vecinos: Vec<Vec<Vecino>>,
}

impl Malla {
    /// Cuadrado de área `area_ha` hectáreas.
    pub fn cuadrada(area_ha: f64, d: f64) -> Malla {
        Malla::cuadrado((area_ha * 10_000.0).sqrt(), d)
    }

    /// Cuadrado de lado `lado` metros.
    pub fn cuadrado(lado: f64, d: f64) -> Malla {
        let h = d * SENO_60;
        let mut posiciones = Vec::new();
        let mut inicio_fila = Vec::new();
        let mut fila = 0;
        loop {
            let y = h / 2.0 + fila as f64 * h;
            if y > lado + HOLGURA_BORDE {
                break;
            }
            inicio_fila.push(posiciones.len());
            let x0 = if fila % 2 == 0 { d / 2.0 } else { d };
            let mut col = 0;
            loop {
                let x = x0 + col as f64 * d;
                if x > lado + HOLGURA_BORDE {
                    break;
                }
                posiciones.push(Posicion { fila, col, x, y });
                col += 1;
            }
            fila += 1;
        }
        inicio_fila.push(posiciones.len());
        Malla {
            d,
            lado,
            posiciones,
            inicio_fila,
        }
    }

    pub fn len(&self) -> usize {
        self.posiciones.len()
    }

    pub fn is_empty(&self) -> bool {
        self.posiciones.is_empty()
    }

    pub fn filas(&self) -> usize {
        self.inicio_fila.len() - 1
    }

    pub fn fila(&self, fila: usize) -> std::ops::Range<usize> {
        self.inicio_fila[fila]..self.inicio_fila[fila + 1]
    }

    pub fn distancia(&self, u: usize, v: usize) -> f64 {
        let (a, b) = (&self.posiciones[u], &self.posiciones[v]);
        (a.x - b.x).hypot(a.y - b.y)
    }

    /// Vecinos de cada posición: los que están a menos de `radio`, y siempre los inmediatos
    /// (δ ≤ d), aunque `radio` sea menor que d. Los anillos son las distancias distintas que
    /// aparecen, agrupadas con tolerancia `tol`.
    pub fn vecindad(&self, radio: f64, tol: f64) -> Vecindad {
        let incluye = |delta: f64| delta <= self.d + tol || delta < radio - tol;
        let alcance = radio.max(self.d) + tol;
        let filas_alcance = (alcance / (self.d * SENO_60)).floor() as usize + 1;

        let mut pares: Vec<(usize, usize, f64)> = Vec::new();
        for u in 0..self.len() {
            let fila_u = self.posiciones[u].fila;
            let ultima = (fila_u + filas_alcance).min(self.filas() - 1);
            for fila_v in fila_u..=ultima {
                for v in self.fila(fila_v) {
                    if v <= u {
                        continue;
                    }
                    let delta = self.distancia(u, v);
                    if incluye(delta) {
                        pares.push((u, v, delta));
                    }
                }
            }
        }

        let mut distancias: Vec<f64> = pares.iter().map(|&(_, _, delta)| delta).collect();
        distancias.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut anillos: Vec<f64> = Vec::new();
        for delta in distancias {
            match anillos.last() {
                Some(&ultimo) if delta - ultimo <= tol => {}
                _ => anillos.push(delta),
            }
        }

        let mut vecinos = vec![Vec::new(); self.len()];
        for (u, v, delta) in pares {
            let anillo = anillos
                .iter()
                .position(|&a| (delta - a).abs() <= tol)
                .expect("toda distancia pertenece a un anillo") as u8;
            vecinos[u].push(Vecino {
                pos: v as u32,
                anillo,
            });
            vecinos[v].push(Vecino {
                pos: u as u32,
                anillo,
            });
        }
        Vecindad { anillos, vecinos }
    }
}

impl Vecindad {
    /// Número de pares no ordenados {u, v} en cada anillo.
    pub fn pares_por_anillo(&self) -> Vec<usize> {
        let mut pares = vec![0; self.anillos.len()];
        for lista in &self.vecinos {
            for vecino in lista {
                pares[vecino.anillo as usize] += 1;
            }
        }
        pares.iter().map(|&dirigidos| dirigidos / 2).collect()
    }

    pub fn max_vecinos(&self) -> usize {
        self.vecinos.iter().map(Vec::len).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const D: f64 = 4.189171180993856;

    #[test]
    fn numero_de_posiciones_por_area() {
        for (area, esperado) in [(1.0, 658), (1.28, 837), (8.0, 5265)] {
            assert_eq!(Malla::cuadrada(area, D).len(), esperado, "área {area}");
        }
    }

    #[test]
    fn posiciones_quedan_dentro_del_cuadrado_a_medio_paso_del_borde() {
        let malla = Malla::cuadrada(1.0, D);
        assert!((malla.lado - 100.0).abs() < 1e-9);
        let min_x = malla.posiciones.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = malla.posiciones.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        assert!((min_x - D / 2.0).abs() < 1e-9);
        assert!((min_y - D * 3f64.sqrt() / 4.0).abs() < 1e-9);
        assert!(malla.posiciones.iter().all(|p| p.x <= 100.0 + 1e-9 && p.y <= 100.0 + 1e-9));
    }

    #[test]
    fn malla_chica_tiene_pares_9_4_2() {
        // d = 1, lado 2.2: 3 filas de 2 puntos; contado a mano: 9 pares a d, 4 a √3, 2 a 2.
        let malla = Malla::cuadrado(2.2, 1.0);
        assert_eq!(malla.len(), 6);
        assert_eq!(malla.filas(), 3);
        let vecindad = malla.vecindad(2.5, 1e-6);
        assert_eq!(vecindad.pares_por_anillo(), vec![9, 4, 2]);
        assert!((vecindad.anillos[1] - 3f64.sqrt()).abs() < 1e-9);
    }

    fn nodo_interior(malla: &Malla) -> usize {
        let centro = malla.lado / 2.0;
        let distancia_al_centro =
            |u: usize| (malla.posiciones[u].x - centro).hypot(malla.posiciones[u].y - centro);
        (0..malla.len())
            .min_by(|&a, &b| distancia_al_centro(a).partial_cmp(&distancia_al_centro(b)).unwrap())
            .unwrap()
    }

    fn conteo_por_anillo(vecindad: &Vecindad, u: usize) -> Vec<usize> {
        let mut conteo = vec![0; vecindad.anillos.len()];
        for vecino in &vecindad.vecinos[u] {
            conteo[vecino.anillo as usize] += 1;
        }
        conteo
    }

    #[test]
    fn nodo_interior_tiene_6_6_6_vecinos_a_d_raiz3d_2d() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(2.5 * D, 1e-6);
        let esperados = [D, 3f64.sqrt() * D, 2.0 * D];
        assert_eq!(vecindad.anillos.len(), 3);
        for (anillo, esperado) in vecindad.anillos.iter().zip(esperados) {
            assert!((anillo - esperado).abs() < 1e-6);
        }
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6, 6, 6]);
        assert_eq!(vecindad.max_vecinos(), 18);
    }

    #[test]
    fn radio_3_125d_agrega_anillos_raiz7_y_3() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(3.125 * D, 1e-6);
        let esperados = [1.0, 3f64.sqrt(), 2.0, 7f64.sqrt(), 3.0];
        assert_eq!(vecindad.anillos.len(), 5);
        for (anillo, esperado) in vecindad.anillos.iter().zip(esperados) {
            assert!((anillo / D - esperado).abs() < 1e-6);
        }
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6, 6, 6, 12, 6]);
    }

    #[test]
    fn radio_menor_a_d_conserva_vecinos_inmediatos() {
        let malla = Malla::cuadrada(1.0, D);
        let vecindad = malla.vecindad(0.75 * D, 1e-6);
        assert_eq!(vecindad.anillos.len(), 1);
        assert_eq!(conteo_por_anillo(&vecindad, nodo_interior(&malla)), vec![6]);
    }

    #[test]
    fn vecindad_es_simetrica_y_anillo_coincide_con_distancia() {
        let malla = Malla::cuadrada(1.28, D);
        let vecindad = malla.vecindad(2.5 * D, 1e-6);
        for (u, lista) in vecindad.vecinos.iter().enumerate() {
            for vecino in lista {
                let v = vecino.pos as usize;
                assert_ne!(u, v);
                assert!(vecindad.vecinos[v]
                    .iter()
                    .any(|w| w.pos as usize == u && w.anillo == vecino.anillo));
                let esperada = vecindad.anillos[vecino.anillo as usize];
                assert!((malla.distancia(u, v) - esperada).abs() < 1e-6);
            }
        }
    }
}
