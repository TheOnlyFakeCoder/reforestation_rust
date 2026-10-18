//! Datos del reto CONAFOR: especies, rasgos, supervivencia y polígonos.
//!
//! Las constantes de este módulo sólo sirven para exportar `data/*.csv` (`reto datos`); el resto
//! del código, en Rust y en Python, lee esos CSV para que todos usen exactamente lo mismo.

use std::error::Error;
use std::path::Path;

pub const N_ESPECIES: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct Especie {
    pub nombre: String,
    /// δ_j: plantas por hectárea en la composición nominal.
    pub delta: u32,
    /// c_j: costo unitario ($).
    pub c: f64,
    /// Supervivencia a 1 y 2 años.
    pub s1: f64,
    pub s2: f64,
    pub familia: String,
    pub genero: String,
    pub forma: String,
    /// 1 superficial, 2 media, 3 profunda.
    pub raiz: u8,
    pub cam: bool,
    pub fija_n: bool,
    /// Altura adulta (m).
    pub altura: f64,
    /// Radio de influencia R_j en múltiplos de d.
    pub radio_d: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Poligono {
    pub id: String,
    /// Numeración del mapa de la socia formadora (la que se usa en reportes).
    pub map_id: u32,
    pub area: f64,
    /// X_j: plantas existentes por especie.
    pub existentes: [u32; N_ESPECIES],
}

impl Poligono {
    pub fn total_existentes(&self) -> u32 {
        self.existentes.iter().sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Datos {
    pub especies: Vec<Especie>,
    pub poligonos: Vec<Poligono>,
}

// (nombre, δ, c, s1, s2)
const ESPECIES: [(&str, u32, f64, f64, f64); N_ESPECIES] = [
    ("Agave lechuguilla", 42, 26.0, 0.9435, 0.9265),
    ("Agave salmiana", 196, 26.0, 0.9587, 0.9082),
    ("Agave scabra", 42, 26.0, 0.8245, 0.7453),
    ("Agave striata", 42, 26.0, 0.9517, 0.9138),
    ("Opuntia cantabrigiensis", 49, 18.0, 0.9272, 0.8287),
    ("Opuntia engelmannii", 38, 21.0, 0.8986, 0.8194),
    ("Opuntia robusta", 73, 18.0, 0.9576, 0.9198),
    ("Opuntia streptacantha", 64, 18.0, 0.8860, 0.8525),
    ("Prosopis laevigata", 86, 26.5, 0.8214, 0.8179),
    ("Yucca filifera", 26, 26.0, 0.8843, 0.7263),
];

// (familia, género, forma, raíz, CAM, fija N, altura m, R_j/d)
type Rasgo = (
    &'static str,
    &'static str,
    &'static str,
    u8,
    bool,
    bool,
    f64,
    f64,
);
const RASGOS: [Rasgo; N_ESPECIES] = [
    ("Asparagaceae", "Agave", "roseta", 1, true, false, 0.5, 1.0),
    ("Asparagaceae", "Agave", "roseta", 1, true, false, 1.8, 1.0),
    ("Asparagaceae", "Agave", "roseta", 1, true, false, 1.2, 1.0),
    ("Asparagaceae", "Agave", "roseta", 1, true, false, 0.8, 1.0),
    (
        "Cactaceae",
        "Opuntia",
        "cladodios",
        1,
        true,
        false,
        1.5,
        1.0,
    ),
    (
        "Cactaceae",
        "Opuntia",
        "cladodios",
        1,
        true,
        false,
        1.2,
        1.0,
    ),
    (
        "Cactaceae",
        "Opuntia",
        "cladodios",
        1,
        true,
        false,
        2.5,
        1.0,
    ),
    (
        "Cactaceae",
        "Opuntia",
        "cladodios",
        1,
        true,
        false,
        3.5,
        2.0,
    ),
    ("Fabaceae", "Prosopis", "árbol", 3, false, true, 8.0, 2.5),
    (
        "Asparagaceae",
        "Yucca",
        "arborescente",
        2,
        true,
        false,
        7.0,
        2.0,
    ),
];

// 6.28 ha es el área de un polígono, no una aproximación de τ.
#[allow(clippy::approx_constant)]
const AREAS: [f64; 30] = [
    1.28, 6.64, 6.76, 1.38, 8.0, 7.82, 5.53, 5.64, 7.11, 6.11, 5.64, 4.92, 5.05, 4.75, 7.97, 7.34,
    5.98, 5.4, 6.28, 7.6, 8.0, 8.0, 7.67, 1.47, 4.19, 7.52, 8.0, 8.0, 7.56, 5.4,
];

const MAP_IDS: [u32; 30] = [
    27, 28, 30, 20, 21, 22, 23, 24, 18, 17, 16, 19, 25, 26, 13, 31, 14, 15, 7, 8, 9, 10, 11, 12, 6,
    2, 3, 4, 5, 1,
];

// Fila = especie, columna = polígono P1..P30 (`plant_matrix` de src/bin/writer.rs).
const EXISTENTES: [[u32; 30]; N_ESPECIES] = [
    [
        8, 58, 66, 10, 65, 67, 41, 32, 58, 47, 48, 36, 50, 35, 40, 70, 56, 45, 56, 69, 54, 75, 44,
        10, 44, 72, 74, 67, 60, 34,
    ],
    [
        46, 263, 236, 52, 280, 306, 209, 252, 269, 209, 233, 182, 189, 186, 311, 290, 233, 204,
        223, 285, 287, 292, 288, 59, 155, 291, 342, 309, 280, 195,
    ],
    [
        16, 47, 51, 15, 66, 63, 43, 46, 58, 47, 41, 43, 49, 38, 52, 55, 39, 34, 51, 57, 70, 60, 69,
        15, 28, 73, 52, 67, 61, 56,
    ],
    [
        16, 49, 50, 9, 61, 61, 49, 40, 53, 41, 37, 44, 28, 43, 62, 54, 46, 33, 48, 75, 62, 68, 51,
        11, 31, 50, 55, 66, 53, 58,
    ],
    [
        11, 60, 71, 15, 92, 81, 48, 73, 57, 66, 48, 51, 43, 39, 68, 84, 71, 56, 50, 77, 89, 104,
        69, 17, 39, 66, 70, 86, 76, 55,
    ],
    [
        14, 44, 56, 10, 54, 62, 52, 39, 58, 43, 40, 41, 33, 44, 63, 50, 45, 46, 48, 74, 58, 62, 60,
        18, 39, 62, 76, 54, 68, 46,
    ],
    [
        18, 111, 100, 18, 124, 118, 87, 86, 94, 96, 94, 68, 73, 73, 132, 105, 83, 82, 93, 107, 126,
        109, 122, 17, 65, 125, 100, 120, 109, 91,
    ],
    [
        12, 95, 78, 20, 114, 91, 60, 79, 91, 71, 65, 72, 76, 61, 89, 77, 81, 48, 90, 103, 109, 95,
        100, 10, 50, 101, 100, 103, 85, 63,
    ],
    [
        15, 98, 123, 23, 106, 133, 97, 91, 117, 108, 94, 67, 97, 74, 149, 121, 97, 95, 120, 104,
        139, 125, 106, 39, 67, 122, 129, 141, 135, 86,
    ],
    [
        9, 39, 28, 11, 41, 45, 27, 26, 37, 29, 27, 19, 25, 23, 53, 40, 32, 17, 33, 25, 44, 43, 39,
        8, 25, 31, 44, 33, 36, 23,
    ],
];

type Resultado<T> = Result<T, Box<dyn Error>>;

impl Datos {
    /// Datos del reto tal como están en `src/bin/writer.rs`, con nombres botánicos corregidos,
    /// rasgos de Gower, radios de influencia y `map_id`.
    pub fn base() -> Datos {
        let especies = ESPECIES
            .iter()
            .zip(RASGOS.iter())
            .map(|(&(nombre, delta, c, s1, s2), rasgo)| {
                let &(familia, genero, forma, raiz, cam, fija_n, altura, radio_d) = rasgo;
                Especie {
                    nombre: nombre.to_string(),
                    delta,
                    c,
                    s1,
                    s2,
                    familia: familia.to_string(),
                    genero: genero.to_string(),
                    forma: forma.to_string(),
                    raiz,
                    cam,
                    fija_n,
                    altura,
                    radio_d,
                }
            })
            .collect();
        let poligonos = (0..AREAS.len())
            .map(|i| Poligono {
                id: format!("P{}", i + 1),
                map_id: MAP_IDS[i],
                area: AREAS[i],
                existentes: std::array::from_fn(|j| EXISTENTES[j][i]),
            })
            .collect();
        Datos {
            especies,
            poligonos,
        }
    }

    /// Σ_j δ_j (658 con los datos del reto).
    pub fn densidad_total(&self) -> u32 {
        self.especies.iter().map(|e| e.delta).sum()
    }

    pub fn poligono_por_map_id(&self, map_id: u32) -> Option<&Poligono> {
        self.poligonos.iter().find(|p| p.map_id == map_id)
    }

    pub fn escribir(&self, dir: &Path) -> Resultado<()> {
        std::fs::create_dir_all(dir)?;

        let mut w = csv::Writer::from_path(dir.join("especies.csv"))?;
        w.write_record(["idx", "especie", "delta", "c"])?;
        for (j, e) in self.especies.iter().enumerate() {
            w.write_record([
                j.to_string(),
                e.nombre.clone(),
                e.delta.to_string(),
                e.c.to_string(),
            ])?;
        }
        w.flush()?;

        let mut w = csv::Writer::from_path(dir.join("supervivencia.csv"))?;
        w.write_record(["idx", "especie", "s0", "s1", "s2"])?;
        for (j, e) in self.especies.iter().enumerate() {
            w.write_record([
                j.to_string(),
                e.nombre.clone(),
                "1".to_string(),
                e.s1.to_string(),
                e.s2.to_string(),
            ])?;
        }
        w.flush()?;

        let mut w = csv::Writer::from_path(dir.join("rasgos.csv"))?;
        w.write_record([
            "idx", "especie", "familia", "genero", "forma", "raiz", "cam", "fija_n", "altura_m",
            "radio_d",
        ])?;
        for (j, e) in self.especies.iter().enumerate() {
            w.write_record([
                j.to_string(),
                e.nombre.clone(),
                e.familia.clone(),
                e.genero.clone(),
                e.forma.clone(),
                e.raiz.to_string(),
                (e.cam as u8).to_string(),
                (e.fija_n as u8).to_string(),
                e.altura.to_string(),
                e.radio_d.to_string(),
            ])?;
        }
        w.flush()?;

        let mut w = csv::Writer::from_path(dir.join("poligonos.csv"))?;
        let mut encabezado = vec![
            "id".to_string(),
            "map_id".to_string(),
            "area_ha".to_string(),
        ];
        encabezado.extend((0..N_ESPECIES).map(|j| format!("X_{j}")));
        w.write_record(&encabezado)?;
        for p in &self.poligonos {
            let mut fila = vec![p.id.clone(), p.map_id.to_string(), p.area.to_string()];
            fila.extend(p.existentes.iter().map(|x| x.to_string()));
            w.write_record(&fila)?;
        }
        w.flush()?;
        Ok(())
    }

    pub fn cargar(dir: &Path) -> Resultado<Datos> {
        let especies_csv = Tabla::leer(&dir.join("especies.csv"))?;
        let supervivencia = Tabla::leer(&dir.join("supervivencia.csv"))?;
        let rasgos = Tabla::leer(&dir.join("rasgos.csv"))?;
        let poligonos_csv = Tabla::leer(&dir.join("poligonos.csv"))?;

        let mut especies = Vec::with_capacity(N_ESPECIES);
        for j in 0..especies_csv.filas.len() {
            for tabla in [&especies_csv, &supervivencia, &rasgos] {
                let idx: usize = tabla.valor(j, "idx")?;
                if idx != j {
                    return Err(
                        format!("{}: la fila {} tiene idx {idx}", tabla.nombre, j + 1).into(),
                    );
                }
            }
            especies.push(Especie {
                nombre: especies_csv.texto(j, "especie")?,
                delta: especies_csv.valor(j, "delta")?,
                c: especies_csv.valor(j, "c")?,
                s1: supervivencia.valor(j, "s1")?,
                s2: supervivencia.valor(j, "s2")?,
                familia: rasgos.texto(j, "familia")?,
                genero: rasgos.texto(j, "genero")?,
                forma: rasgos.texto(j, "forma")?,
                raiz: rasgos.valor(j, "raiz")?,
                cam: rasgos.valor::<u8>(j, "cam")? == 1,
                fija_n: rasgos.valor::<u8>(j, "fija_n")? == 1,
                altura: rasgos.valor(j, "altura_m")?,
                radio_d: rasgos.valor(j, "radio_d")?,
            });
        }
        if especies.len() != N_ESPECIES {
            return Err(format!("se esperaban {N_ESPECIES} especies en {}", dir.display()).into());
        }

        let mut poligonos = Vec::with_capacity(poligonos_csv.filas.len());
        for i in 0..poligonos_csv.filas.len() {
            let mut existentes = [0u32; N_ESPECIES];
            for (j, x) in existentes.iter_mut().enumerate() {
                *x = poligonos_csv.valor(i, &format!("X_{j}"))?;
            }
            poligonos.push(Poligono {
                id: poligonos_csv.texto(i, "id")?,
                map_id: poligonos_csv.valor(i, "map_id")?,
                area: poligonos_csv.valor(i, "area_ha")?,
                existentes,
            });
        }

        Ok(Datos {
            especies,
            poligonos,
        })
    }
}

/// CSV leído completo, con acceso a celdas por nombre de columna.
struct Tabla {
    nombre: String,
    encabezado: csv::StringRecord,
    filas: Vec<csv::StringRecord>,
}

impl Tabla {
    fn leer(path: &Path) -> Resultado<Tabla> {
        let mut lector =
            csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let encabezado = lector.headers()?.clone();
        let filas = lector.records().collect::<Result<Vec<_>, _>>()?;
        Ok(Tabla {
            nombre: path.display().to_string(),
            encabezado,
            filas,
        })
    }

    fn texto(&self, fila: usize, columna: &str) -> Resultado<String> {
        let indice = self
            .encabezado
            .iter()
            .position(|h| h == columna)
            .ok_or_else(|| format!("falta la columna {columna} en {}", self.nombre))?;
        let registro = self
            .filas
            .get(fila)
            .ok_or_else(|| format!("{}: falta la fila {}", self.nombre, fila + 1))?;
        Ok(registro.get(indice).unwrap_or("").trim().to_string())
    }

    fn valor<T: std::str::FromStr>(&self, fila: usize, columna: &str) -> Resultado<T> {
        let texto = self.texto(fila, columna)?;
        texto.parse().map_err(|_| {
            format!(
                "{}: valor inválido '{texto}' en la columna {columna}, fila {}",
                self.nombre,
                fila + 1
            )
            .into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dir_temporal(nombre: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("reto_{}_{}", nombre, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn base_tiene_densidad_658_y_area_183_01() {
        let datos = Datos::base();
        assert_eq!(datos.densidad_total(), 658);
        let area: f64 = datos.poligonos.iter().map(|p| p.area).sum();
        assert!((area - 183.01).abs() < 1e-9);
        assert_eq!(datos.poligonos.len(), 30);
    }

    #[test]
    fn map_id_de_p1_es_27_y_falta_el_29() {
        let datos = Datos::base();
        assert_eq!(datos.poligonos[0].id, "P1");
        assert_eq!(datos.poligonos[0].map_id, 27);
        assert_eq!(datos.poligonos[29].map_id, 1);
        let mut ids: Vec<u32> = datos.poligonos.iter().map(|p| p.map_id).collect();
        ids.sort();
        let esperados: Vec<u32> = (1..=31).filter(|&i| i != 29).collect();
        assert_eq!(ids, esperados);
        assert_eq!(datos.poligono_por_map_id(27).unwrap().id, "P1");
        assert!(datos.poligono_por_map_id(29).is_none());
    }

    #[test]
    fn existentes_de_p1_y_p30_coinciden_con_writer() {
        let datos = Datos::base();
        assert_eq!(
            datos.poligonos[0].existentes,
            [8, 46, 16, 16, 11, 14, 18, 12, 15, 9]
        );
        assert_eq!(datos.poligonos[0].total_existentes(), 165);
        assert_eq!(
            datos.poligonos[29].existentes,
            [34, 195, 56, 58, 55, 46, 91, 63, 86, 23]
        );
    }

    #[test]
    fn nombres_botanicos_corregidos() {
        let datos = Datos::base();
        assert_eq!(datos.especies[5].nombre, "Opuntia engelmannii");
        assert_eq!(datos.especies[7].nombre, "Opuntia streptacantha");
    }

    #[test]
    fn escribir_y_cargar_conserva_los_datos() {
        let dir = dir_temporal("roundtrip");
        let datos = Datos::base();
        datos.escribir(&dir).unwrap();
        let leidos = Datos::cargar(&dir).unwrap();
        assert_eq!(leidos, datos);
    }

    #[test]
    fn cargar_falla_si_falta_un_archivo() {
        let dir = dir_temporal("falta");
        Datos::base().escribir(&dir).unwrap();
        std::fs::remove_file(dir.join("rasgos.csv")).unwrap();
        assert!(Datos::cargar(&dir).is_err());
    }

    #[test]
    fn cargar_falla_si_falta_una_columna() {
        let dir = dir_temporal("columna");
        Datos::base().escribir(&dir).unwrap();
        std::fs::write(dir.join("especies.csv"), "idx,especie,c\n0,A,26\n").unwrap();
        let error = Datos::cargar(&dir).unwrap_err().to_string();
        assert!(error.contains("delta"), "{error}");
    }
}
