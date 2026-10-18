//! Malla tresbolillo (hexagonal) con coordenadas en metros.
//!
//! El modelo usa `Malla::para_objetivo`: un rectángulo casi cuadrado (ancho/alto ∈ [0.8, 1.25])
//! con |V| ≥ round(658·A) y excedente mínimo, de modo que D_tot = round(658·A). Los centros
//! quedan al menos a d/2 del borde: x = d/2 + c·d en filas pares y x = d + c·d en impares;
//! y = h/2 + f·h con h = d·√3/2.

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

/// Cómo se reparten las posiciones por fila en un rectángulo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposicion {
    /// c posiciones en todas las filas; ancho (c + ½)·d.
    Completa,
    /// c posiciones en filas pares y c − 1 en impares; ancho c·d.
    Alterna,
}

#[derive(Clone, Debug)]
pub struct Malla {
    pub d: f64,
    /// LEGACY: sólo lo llenan `cuadrada`/`cuadrado`; en mallas rectangulares vale 0. Usar
    /// `ancho` y `alto`.
    pub lado: f64,
    pub ancho: f64,
    pub alto: f64,
    /// Posiciones en las filas pares.
    pub columnas: usize,
    pub disposicion: Disposicion,
    pub posiciones: Vec<Posicion>,
    /// Índice de la primera posición de cada fila; el último elemento es `len()`.
    inicio_fila: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vecino {
    pub pos: u32,
    pub anillo: u16,
}

/// Vecinos de cada posición agrupados por anillo (distancia distinta en la malla).
#[derive(Clone, Debug)]
pub struct Vecindad {
    /// Distancia de cada anillo (m), en orden creciente.
    pub anillos: Vec<f64>,
    pub vecinos: Vec<Vec<Vecino>>,
}

impl Malla {
    /// LEGACY: cuadrado de área `area_ha` hectáreas. Pierde hasta 2.7 % de posiciones por el borde
    /// (map 12: 941 contra 967); reemplazado por `para_objetivo`.
    pub fn cuadrada(area_ha: f64, d: f64) -> Malla {
        Malla::cuadrado((area_ha * 10_000.0).sqrt(), d)
    }

    /// LEGACY: cuadrado de lado `lado` metros (ver `cuadrada`). Es el único constructor que llena
    /// `lado`; `disposicion` se reporta como Alterna aunque las filas tengan igual largo.
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
            ancho: lado,
            alto: lado,
            columnas: inicio_fila.get(1).copied().unwrap_or(0),
            disposicion: Disposicion::Alterna,
            posiciones,
            inicio_fila,
        }
    }

    /// Rectángulo de `filas` filas con `columnas` posiciones en las filas pares; las impares
    /// tienen c (Completa) o c − 1 (Alterna). Los centros quedan a d/2 del borde izquierdo en
    /// filas pares, a d en impares, y a h/2 de los bordes inferior y superior.
    pub fn rectangular(columnas: usize, filas: usize, disposicion: Disposicion, d: f64) -> Malla {
        let (en_impares, ancho) = match disposicion {
            Disposicion::Completa => (columnas, (columnas as f64 + 0.5) * d),
            Disposicion::Alterna => (columnas - 1, columnas as f64 * d),
        };
        let h = d * SENO_60;
        let mut posiciones = Vec::new();
        let mut inicio_fila = Vec::with_capacity(filas + 1);
        for fila in 0..filas {
            inicio_fila.push(posiciones.len());
            let y = h / 2.0 + fila as f64 * h;
            let (x0, en_fila) = if fila % 2 == 0 {
                (d / 2.0, columnas)
            } else {
                (d, en_impares)
            };
            for col in 0..en_fila {
                posiciones.push(Posicion {
                    fila,
                    col,
                    x: x0 + col as f64 * d,
                    y,
                });
            }
        }
        inicio_fila.push(posiciones.len());
        Malla {
            d,
            lado: 0.0,
            ancho,
            alto: filas as f64 * h,
            columnas,
            disposicion,
            posiciones,
            inicio_fila,
        }
    }

    /// Rectángulo casi cuadrado con |V| ≥ `objetivo` y el menor excedente posible, con
    /// ancho/alto ∈ [0.8, 1.25]. Desempates: proporción más cercana a 1, menos columnas, menos
    /// filas, Alterna antes que Completa.
    pub fn para_objetivo(objetivo: usize, d: f64) -> Malla {
        let h = d * SENO_60;
        let limite = ((objetivo as f64).sqrt() * 2.0) as usize + 3;
        let mut mejor: Option<(usize, f64, usize, usize, Disposicion)> = None;
        for columnas in 2..=limite {
            for filas in 2..=limite {
                for disposicion in [Disposicion::Alterna, Disposicion::Completa] {
                    let (n_pos, ancho) = match disposicion {
                        Disposicion::Completa => (columnas * filas, (columnas as f64 + 0.5) * d),
                        Disposicion::Alterna => (columnas * filas - filas / 2, columnas as f64 * d),
                    };
                    let proporcion = ancho / (filas as f64 * h);
                    if n_pos < objetivo || !(0.8..=1.25).contains(&proporcion) {
                        continue;
                    }
                    let candidato = (
                        n_pos - objetivo,
                        (proporcion - 1.0).abs(),
                        columnas,
                        filas,
                        disposicion,
                    );
                    let es_mejor = match &mejor {
                        None => true,
                        Some(m) => {
                            (candidato.0, candidato.1, candidato.2, candidato.3)
                                .partial_cmp(&(m.0, m.1, m.2, m.3))
                                == Some(std::cmp::Ordering::Less)
                        }
                    };
                    if es_mejor {
                        mejor = Some(candidato);
                    }
                }
            }
        }
        let (_, _, columnas, filas, disposicion) =
            mejor.expect("siempre existe un rectángulo con |V| ≥ objetivo");
        Malla::rectangular(columnas, filas, disposicion, d)
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
                .expect("toda distancia pertenece a un anillo");
            let anillo = u16::try_from(anillo).expect("más de 65 535 anillos");
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
        let min_x = malla
            .posiciones
            .iter()
            .map(|p| p.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = malla
            .posiciones
            .iter()
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        assert!((min_x - D / 2.0).abs() < 1e-9);
        assert!((min_y - D * 3f64.sqrt() / 4.0).abs() < 1e-9);
        assert!(malla
            .posiciones
            .iter()
            .all(|p| p.x <= 100.0 + 1e-9 && p.y <= 100.0 + 1e-9));
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
            .min_by(|&a, &b| {
                distancia_al_centro(a)
                    .partial_cmp(&distancia_al_centro(b))
                    .unwrap()
            })
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
        assert_eq!(
            conteo_por_anillo(&vecindad, nodo_interior(&malla)),
            vec![6, 6, 6]
        );
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
        assert_eq!(
            conteo_por_anillo(&vecindad, nodo_interior(&malla)),
            vec![6, 6, 6, 12, 6]
        );
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

    #[test]
    fn rectangular_alterna_tiene_c_y_c_menos_1_por_fila() {
        let malla = Malla::rectangular(3, 3, Disposicion::Alterna, 1.0);
        assert_eq!(malla.len(), 3 + 2 + 3);
        assert_eq!(malla.filas(), 3);
        assert!((malla.ancho - 3.0).abs() < 1e-12);
        assert!((malla.alto - 3.0 * SENO_60).abs() < 1e-12);
    }

    #[test]
    fn rectangular_completa_tiene_c_por_fila_y_ancho_c_mas_medio() {
        let malla = Malla::rectangular(3, 3, Disposicion::Completa, 1.0);
        assert_eq!(malla.len(), 9);
        assert!((malla.ancho - 3.5).abs() < 1e-12);
        assert!((malla.alto - 3.0 * SENO_60).abs() < 1e-12);
    }

    #[test]
    fn para_objetivo_da_excedente_minimo_con_proporcion_valida() {
        // Calculado con una búsqueda independiente en Python: (objetivo, |V|, columnas, filas).
        let casos = [
            (658, 658, 24, 28, Disposicion::Alterna),
            (842, 842, 26, 33, Disposicion::Alterna),
            (967, 969, 29, 34, Disposicion::Alterna),
            (4132, 4140, 60, 69, Disposicion::Completa),
        ];
        for (objetivo, n_pos, columnas, filas, disposicion) in casos {
            let malla = Malla::para_objetivo(objetivo, D);
            assert_eq!(
                (
                    malla.len(),
                    malla.columnas,
                    malla.filas(),
                    malla.disposicion
                ),
                (n_pos, columnas, filas, disposicion),
                "objetivo {objetivo}"
            );
            let proporcion = malla.ancho / malla.alto;
            assert!(
                (0.8..=1.25).contains(&proporcion),
                "objetivo {objetivo}: {proporcion}"
            );
            let h = D * SENO_60;
            for p in &malla.posiciones {
                assert!(p.x >= D / 2.0 - 1e-9 && p.x <= malla.ancho - D / 2.0 + 1e-9);
                assert!(p.y >= h / 2.0 - 1e-9 && p.y <= malla.alto - h / 2.0 + 1e-9);
            }
        }
    }

    #[test]
    fn mas_de_255_anillos_conservan_su_distancia() {
        // 40 × 40 alterna con d = 1: 346 distancias distintas menores que 35.8 (contado en Python;
        // la distancia de la red más cercana a 35.8 está a 0.009, lejos de la tolerancia).
        let malla = Malla::rectangular(40, 40, Disposicion::Alterna, 1.0);
        let vecindad = malla.vecindad(35.8, 1e-6);
        assert_eq!(vecindad.anillos.len(), 346);
        for (u, lista) in vecindad.vecinos.iter().enumerate() {
            for vecino in lista {
                let esperada = vecindad.anillos[vecino.anillo as usize];
                assert!((malla.distancia(u, vecino.pos as usize) - esperada).abs() < 1e-6);
            }
        }
    }
}
