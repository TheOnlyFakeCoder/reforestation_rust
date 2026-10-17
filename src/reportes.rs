//! Tablas de resultados que se exportan a `results/`.

use crate::competencia::{alcance, radio_max, radios};
use crate::composicion::d_tot;
use crate::config::Config;
use crate::datos::{Datos, Especie};
use crate::malla::{Disposicion, Malla};
use std::error::Error;
use std::path::Path;

/// Resumen de la malla de una instancia (1 ha o un polígono).
#[derive(Clone, Debug, PartialEq)]
pub struct FilaMalla {
    pub instancia: String,
    pub map_id: Option<u32>,
    pub area: f64,
    pub columnas: usize,
    pub filas: usize,
    pub disposicion: Disposicion,
    pub ancho_m: f64,
    pub alto_m: f64,
    /// |V|
    pub n_pos: usize,
    /// round(658·A)
    pub objetivo: usize,
    /// |V| − round(658·A): posiciones que quedan vacías.
    pub excedente: usize,
    pub d_tot: usize,
    pub anillos_m: Vec<f64>,
    pub pares: Vec<usize>,
}

/// Matriz W con nombres de especie en la primera fila y la primera columna.
pub fn escribir_w(path: &Path, especies: &[Especie], w: &[Vec<f64>]) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado = vec!["especie".to_string()];
    encabezado.extend(especies.iter().map(|e| e.nombre.clone()));
    csv.write_record(&encabezado)?;
    for (especie, fila) in especies.iter().zip(w) {
        let mut registro = vec![especie.nombre.clone()];
        registro.extend(fila.iter().map(|x| x.to_string()));
        csv.write_record(&registro)?;
    }
    csv.flush()?;
    Ok(())
}

/// Una fila por instancia: rectángulo (columnas, filas, disposición, ancho y alto en m), |V|,
/// round(658·A), excedente, D_tot y pares por anillo.
pub fn escribir_resumen_malla(path: &Path, filas: &[FilaMalla]) -> Result<(), Box<dyn Error>> {
    let n_anillos = filas.iter().map(|f| f.pares.len()).max().unwrap_or(0);
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado: Vec<String> = [
        "instancia",
        "map_id",
        "area_ha",
        "columnas",
        "filas",
        "disposicion",
        "ancho_m",
        "alto_m",
        "n_pos",
        "round_658A",
        "excedente",
        "d_tot",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    encabezado.extend((1..=n_anillos).map(|k| format!("pares_anillo_{k}")));
    csv.write_record(&encabezado)?;
    for fila in filas {
        let mut registro = vec![
            fila.instancia.clone(),
            fila.map_id.map(|m| m.to_string()).unwrap_or_default(),
            fila.area.to_string(),
            fila.columnas.to_string(),
            fila.filas.to_string(),
            match fila.disposicion {
                Disposicion::Completa => "completa".to_string(),
                Disposicion::Alterna => "alterna".to_string(),
            },
            fila.ancho_m.to_string(),
            fila.alto_m.to_string(),
            fila.n_pos.to_string(),
            fila.objetivo.to_string(),
            fila.excedente.to_string(),
            fila.d_tot.to_string(),
        ];
        registro.extend(
            (0..n_anillos).map(|k| fila.pares.get(k).map(|p| p.to_string()).unwrap_or_default()),
        );
        csv.write_record(&registro)?;
    }
    csv.flush()?;
    Ok(())
}

/// Formato largo: una fila por instancia y anillo, con la distancia en m y en múltiplos de d.
pub fn escribir_anillos(path: &Path, filas: &[FilaMalla], d: f64) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    csv.write_record([
        "instancia",
        "map_id",
        "anillo",
        "distancia_m",
        "distancia_sobre_d",
        "pares",
    ])?;
    for fila in filas {
        for (k, (&distancia, &pares)) in fila.anillos_m.iter().zip(&fila.pares).enumerate() {
            csv.write_record([
                fila.instancia.clone(),
                fila.map_id.map(|m| m.to_string()).unwrap_or_default(),
                (k + 1).to_string(),
                distancia.to_string(),
                (distancia / d).to_string(),
                pares.to_string(),
            ])?;
        }
    }
    csv.flush()?;
    Ok(())
}

/// Factor de alcance h para cada par de clases de radio (R_j/d con el factor aplicado) y cada
/// anillo de la malla.
pub fn escribir_alcance(
    path: &Path,
    especies: &[Especie],
    config: &Config,
    anillos: &[f64],
) -> Result<(), Box<dyn Error>> {
    let d = config.d;
    let mut clases: Vec<f64> = radios(especies, d, config.factor_radios)
        .iter()
        .map(|r| r / d)
        .collect();
    clases.sort_by(|a, b| a.partial_cmp(b).unwrap());
    clases.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

    let mut csv = csv::Writer::from_path(path)?;
    csv.write_record(["R_j_d", "R_k_d", "R_jk_d", "anillo", "delta_sobre_d", "h"])?;
    for (i, &r_j) in clases.iter().enumerate() {
        for &r_k in &clases[i..] {
            let r_jk = r_j.max(r_k);
            for (k, &delta) in anillos.iter().enumerate() {
                let h = alcance(delta, d, r_jk * d, config.tolerancia);
                csv.write_record([
                    r_j.to_string(),
                    r_k.to_string(),
                    r_jk.to_string(),
                    (k + 1).to_string(),
                    (delta / d).to_string(),
                    h.to_string(),
                ])?;
            }
        }
    }
    csv.flush()?;
    Ok(())
}

/// Malla de 1 ha seguida de los polígonos en su orden (P1..P30), identificados por `map_id`.
pub fn resumen_malla(datos: &Datos, config: &Config) -> Vec<FilaMalla> {
    let densidad = datos.densidad_total();
    let radio = radio_max(
        &radios(&datos.especies, config.d, config.factor_radios),
        config.d,
    );
    let mut instancias = vec![("1ha".to_string(), None, 1.0)];
    instancias.extend(
        datos
            .poligonos
            .iter()
            .map(|p| (format!("map_{}", p.map_id), Some(p.map_id), p.area)),
    );
    instancias
        .into_iter()
        .map(|(instancia, map_id, area)| {
            let objetivo = (densidad as f64 * area).round() as usize;
            let malla = Malla::para_objetivo(objetivo, config.d);
            let vecindad = malla.vecindad(radio, config.tolerancia);
            FilaMalla {
                instancia,
                map_id,
                area,
                columnas: malla.columnas,
                filas: malla.filas(),
                disposicion: malla.disposicion,
                ancho_m: malla.ancho,
                alto_m: malla.alto,
                n_pos: malla.len(),
                objetivo,
                excedente: malla.len() - objetivo,
                d_tot: d_tot(malla.len(), area, densidad),
                pares: vecindad.pares_por_anillo(),
                anillos_m: vecindad.anillos,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumen_malla_1ha_y_p1() {
        let filas = resumen_malla(&Datos::base(), &Config::default());
        assert_eq!(filas.len(), 31);
        let primera = &filas[0];
        assert_eq!(
            (
                primera.instancia.as_str(),
                primera.columnas,
                primera.filas,
                primera.n_pos,
                primera.objetivo,
                primera.excedente,
                primera.d_tot
            ),
            ("1ha", 24, 28, 658, 658, 0, 658)
        );
        let p1 = filas.iter().find(|f| f.map_id == Some(27)).unwrap();
        assert_eq!(
            (p1.columnas, p1.filas, p1.n_pos, p1.objetivo, p1.excedente, p1.d_tot),
            (26, 33, 842, 842, 0, 842)
        );
        assert!((p1.ancho_m - 26.0 * crate::config::Config::default().d).abs() < 1e-9);
        assert_eq!(p1.pares.len(), 3);
        assert_eq!(p1.anillos_m.len(), 3);
        let map12 = filas.iter().find(|f| f.map_id == Some(12)).unwrap();
        assert_eq!((map12.n_pos, map12.objetivo, map12.excedente, map12.d_tot), (969, 967, 2, 967));
    }

    fn dir_temporal(nombre: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("reto_rep_{}_{}", nombre, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn escribir_w_deja_matriz_con_nombres() {
        let especies = Datos::base().especies;
        let w = crate::competencia::matriz_w(&especies, 0.6);
        let path = dir_temporal("w").join("W.csv");
        escribir_w(&path, &especies, &w).unwrap();
        let texto = std::fs::read_to_string(&path).unwrap();
        let filas: Vec<Vec<&str>> = texto.lines().map(|l| l.split(',').collect()).collect();
        assert_eq!(filas.len(), 11);
        assert_eq!(filas[0][0], "especie");
        assert_eq!(filas[0][9], "Prosopis laevigata");
        assert_eq!(filas[1][0], "Agave lechuguilla");
        assert_eq!(filas[1][1].parse::<f64>().unwrap(), 1.0);
        assert_eq!(filas[1][9].parse::<f64>().unwrap(), 0.0);
        assert!((filas[10][9].parse::<f64>().unwrap() - w[9][8]).abs() < 1e-12);
    }

    #[test]
    fn escribir_resumen_malla_una_fila_por_instancia() {
        let filas = resumen_malla(&Datos::base(), &Config::default());
        let path = dir_temporal("resumen").join("malla_resumen.csv");
        escribir_resumen_malla(&path, &filas).unwrap();
        let texto = std::fs::read_to_string(&path).unwrap();
        let lineas: Vec<&str> = texto.lines().collect();
        assert_eq!(lineas.len(), 32);
        assert_eq!(
            lineas[0],
            "instancia,map_id,area_ha,columnas,filas,disposicion,ancho_m,alto_m,n_pos,round_658A,\
             excedente,d_tot,pares_anillo_1,pares_anillo_2,pares_anillo_3"
        );
        let p1: Vec<&str> = lineas[2].split(',').collect();
        assert_eq!(&p1[..6], ["map_27", "27", "1.28", "26", "33", "alterna"]);
        assert!((p1[6].parse::<f64>().unwrap() - 108.918).abs() < 1e-3);
        assert_eq!(&p1[8..12], ["842", "842", "0", "842"]);
    }

    #[test]
    fn escribir_alcance_tabula_h_por_par_de_radios_y_anillo() {
        let config = Config::default();
        let d = config.d;
        let anillos = [d, 3f64.sqrt() * d, 2.0 * d];
        let path = dir_temporal("alcance").join("alcance_h.csv");
        escribir_alcance(&path, &Datos::base().especies, &config, &anillos).unwrap();
        let texto = std::fs::read_to_string(&path).unwrap();
        let lineas: Vec<Vec<&str>> = texto.lines().map(|l| l.split(',').collect()).collect();
        assert_eq!(lineas[0], ["R_j_d", "R_k_d", "R_jk_d", "anillo", "delta_sobre_d", "h"]);
        // Clases de radio 1, 2 y 2.5 (·d): 6 pares × 3 anillos.
        assert_eq!(lineas.len(), 1 + 6 * 3);
        let h = |rj: &str, rk: &str, anillo: &str| -> f64 {
            lineas
                .iter()
                .find(|l| l[0] == rj && l[1] == rk && l[3] == anillo)
                .unwrap_or_else(|| panic!("falta ({rj}, {rk}, {anillo})"))[5]
                .parse()
                .unwrap()
        };
        assert_eq!(h("1", "1", "1"), 1.0);
        assert_eq!(h("1", "1", "2"), 0.0);
        assert!((h("2", "2", "2") - 0.26795).abs() < 1e-4);
        assert_eq!(h("2", "2", "3"), 0.0);
        assert!((h("1", "2.5", "2") - 0.51197).abs() < 1e-4);
        assert!((h("2.5", "2.5", "3") - 0.33333).abs() < 1e-4);
    }

    #[test]
    fn escribir_anillos_una_fila_por_instancia_y_anillo() {
        let config = Config::default();
        let filas = resumen_malla(&Datos::base(), &config);
        let path = dir_temporal("anillos").join("malla_anillos.csv");
        escribir_anillos(&path, &filas[..2], config.d).unwrap();
        let texto = std::fs::read_to_string(&path).unwrap();
        let lineas: Vec<Vec<&str>> = texto.lines().map(|l| l.split(',').collect()).collect();
        assert_eq!(lineas.len(), 7);
        assert_eq!(
            lineas[0],
            ["instancia", "map_id", "anillo", "distancia_m", "distancia_sobre_d", "pares"]
        );
        assert_eq!(&lineas[2][..3], ["1ha", "", "2"]);
        let sobre_d: f64 = lineas[2][4].parse().unwrap();
        assert!((sobre_d - 3f64.sqrt()).abs() < 1e-6);
        assert_eq!(lineas[2][5], filas[0].pares[1].to_string());
        assert_eq!(&lineas[6][..3], ["map_27", "27", "3"]);
    }
}
