//! Tablas de resultados que se exportan a `results/`.

use crate::busqueda_tabu::MotivoParo;
use crate::competencia::{alcance, radio_max, radios};
use crate::composicion::d_tot;
use crate::config::Config;
use crate::datos::{Datos, Especie, N_ESPECIES};
use crate::escenarios::estadisticos;
use crate::malla::{Disposicion, Malla};
use crate::paso1::{mejor_fila, FilaPaso1, Instancia};
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

// ---------------------------------------------------------------------------------------------
// Paso 1
// ---------------------------------------------------------------------------------------------

fn columnas(prefijo: &str) -> Vec<String> {
    (0..N_ESPECIES).map(|j| format!("{prefijo}_{j}")).collect()
}

fn media(xs: impl Iterator<Item = f64>) -> f64 {
    let (suma, n) = xs.fold((0.0, 0usize), |(s, n), x| (s + x, n + 1));
    suma / n as f64
}

/// Una fila por instancia × γ × semilla.
pub fn escribir_barrido_gamma(path: &Path, filas: &[FilaPaso1]) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado: Vec<String> = ["instancia", "map_id", "gamma", "semilla"]
        .map(String::from)
        .to_vec();
    encabezado.extend(columnas("D"));
    encabezado.extend(
        [
            "d_tot",
            "suma_theta_d",
            "suma_c_d",
            "z_paso1",
            "z_opt",
            "z_opt_por_planta",
            "tiempo_paso1_s",
            "iter_paso1",
            "tiempo_opt_s",
            "iter_opt",
            "motivo_paro_paso1",
            "motivo_paro_opt",
        ]
        .map(String::from),
    );
    csv.write_record(&encabezado)?;
    for f in filas {
        let mut r = vec![
            f.instancia.clone(),
            f.map_id.to_string(),
            f.gamma.to_string(),
            f.semilla.to_string(),
        ];
        r.extend(f.d.iter().map(|x| x.to_string()));
        r.extend([
            f.d_tot.to_string(),
            f.suma_theta_d.to_string(),
            f.suma_c_d.to_string(),
            f.z_paso1.to_string(),
            f.z_opt.to_string(),
            f.z_opt_por_planta.to_string(),
            f.tiempo_paso1_s.to_string(),
            f.iter_paso1.to_string(),
            f.tiempo_opt_s.to_string(),
            f.iter_opt.to_string(),
            f.motivo_paro_paso1.como_str().to_string(),
            f.motivo_paro_opt.como_str().to_string(),
        ]);
        csv.write_record(&r)?;
    }
    csv.flush()?;
    Ok(())
}

/// Una fila por instancia × γ: D de la semilla con menor objetivo del paso 1 y estadísticos de
/// Z_opt, tiempo e iteraciones sobre las semillas.
pub fn escribir_resumen_gamma(path: &Path, filas: &[FilaPaso1]) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado: Vec<String> = ["instancia", "map_id", "gamma"].map(String::from).to_vec();
    encabezado.extend(columnas("D"));
    encabezado.extend(
        [
            "suma_theta_d",
            "suma_c_d",
            "z_opt_media",
            "z_opt_min",
            "z_opt_max",
            "z_opt_por_planta_media",
            "tiempo_medio_s",
            "iter_media",
            "z_opt_sd",
            "paros_tope",
            "paros_sin_mejora",
        ]
        .map(String::from),
    );
    csv.write_record(&encabezado)?;
    let mut grupos: Vec<(&str, f64)> = Vec::new();
    for f in filas {
        if !grupos
            .iter()
            .any(|&(i, g)| i == f.instancia && g == f.gamma)
        {
            grupos.push((&f.instancia, f.gamma));
        }
    }
    for (instancia, gamma) in grupos {
        let grupo = || {
            filas
                .iter()
                .filter(move |f| f.instancia == instancia && f.gamma == gamma)
        };
        let mejor = mejor_fila(grupo()).expect("grupo no vacío");
        let z_opt: Vec<f64> = grupo().map(|f| f.z_opt).collect();
        let z_opt_sd = estadisticos(&z_opt).1;
        let paros = |m: MotivoParo| {
            grupo()
                .map(|f| {
                    [f.motivo_paro_paso1, f.motivo_paro_opt]
                        .iter()
                        .filter(|&&x| x == m)
                        .count()
                })
                .sum::<usize>()
        };
        let mut r = vec![
            instancia.to_string(),
            mejor.map_id.to_string(),
            gamma.to_string(),
        ];
        r.extend(mejor.d.iter().map(|x| x.to_string()));
        r.extend([
            mejor.suma_theta_d.to_string(),
            mejor.suma_c_d.to_string(),
            media(grupo().map(|f| f.z_opt)).to_string(),
            grupo()
                .map(|f| f.z_opt)
                .fold(f64::INFINITY, f64::min)
                .to_string(),
            grupo()
                .map(|f| f.z_opt)
                .fold(f64::NEG_INFINITY, f64::max)
                .to_string(),
            media(grupo().map(|f| f.z_opt_por_planta)).to_string(),
            media(grupo().map(|f| f.tiempo_paso1_s + f.tiempo_opt_s)).to_string(),
            media(grupo().map(|f| (f.iter_paso1 + f.iter_opt) as f64)).to_string(),
            z_opt_sd.to_string(),
            paros(MotivoParo::Tope).to_string(),
            paros(MotivoParo::SinMejora).to_string(),
        ]);
        csv.write_record(&r)?;
    }
    csv.flush()?;
    Ok(())
}

/// Composición D* de γ* de cada instancia con D̄, L y U.
pub fn escribir_composicion_paso1(
    path: &Path,
    elegidas: &[(&Instancia, &FilaPaso1)],
) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado: Vec<String> = ["instancia", "map_id", "area_ha", "d_tot"]
        .map(String::from)
        .to_vec();
    for prefijo in ["D", "Dnom", "L", "U"] {
        encabezado.extend(columnas(prefijo));
    }
    csv.write_record(&encabezado)?;
    for (instancia, fila) in elegidas {
        let mut r = vec![
            instancia.nombre.clone(),
            instancia.map_id.to_string(),
            instancia.area.to_string(),
            instancia.d_tot.to_string(),
        ];
        r.extend(fila.d.iter().map(|x| x.to_string()));
        r.extend(instancia.nominal.iter().map(|x| x.to_string()));
        r.extend(instancia.banda.l.iter().map(|x| x.to_string()));
        r.extend(instancia.banda.u.iter().map(|x| x.to_string()));
        csv.write_record(&r)?;
    }
    csv.flush()?;
    Ok(())
}

/// Patrón φ*: una fila por posición; especie −1 si está vacía.
pub fn escribir_patron(path: &Path, malla: &Malla, phi: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    csv.write_record(["pos", "fila", "col", "x", "y", "especie"])?;
    for (u, p) in malla.posiciones.iter().enumerate() {
        csv.write_record([
            u.to_string(),
            p.fila.to_string(),
            p.col.to_string(),
            p.x.to_string(),
            p.y.to_string(),
            (phi[u] as i32 - 1).to_string(),
        ])?;
    }
    csv.flush()?;
    Ok(())
}

/// Convergencia del paso 1: objetivo actual y mejor por iteración.
pub fn escribir_convergencia(path: &Path, fila: &FilaPaso1) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    csv.write_record(["iteracion", "actual", "mejor"])?;
    for (k, (actual, mejor)) in fila.traza.iter().zip(&fila.curva).enumerate() {
        csv.write_record([(k + 1).to_string(), actual.to_string(), mejor.to_string()])?;
    }
    csv.flush()?;
    Ok(())
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
            (
                p1.columnas,
                p1.filas,
                p1.n_pos,
                p1.objetivo,
                p1.excedente,
                p1.d_tot
            ),
            (26, 33, 842, 842, 0, 842)
        );
        assert!((p1.ancho_m - 26.0 * crate::config::Config::default().d).abs() < 1e-9);
        assert_eq!(p1.pares.len(), 3);
        assert_eq!(p1.anillos_m.len(), 3);
        let map12 = filas.iter().find(|f| f.map_id == Some(12)).unwrap();
        assert_eq!(
            (map12.n_pos, map12.objetivo, map12.excedente, map12.d_tot),
            (969, 967, 2, 967)
        );
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
        assert_eq!(
            lineas[0],
            ["R_j_d", "R_k_d", "R_jk_d", "anillo", "delta_sobre_d", "h"]
        );
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
            [
                "instancia",
                "map_id",
                "anillo",
                "distancia_m",
                "distancia_sobre_d",
                "pares"
            ]
        );
        assert_eq!(&lineas[2][..3], ["1ha", "", "2"]);
        let sobre_d: f64 = lineas[2][4].parse().unwrap();
        assert!((sobre_d - 3f64.sqrt()).abs() < 1e-6);
        assert_eq!(lineas[2][5], filas[0].pares[1].to_string());
        assert_eq!(&lineas[6][..3], ["map_27", "27", "3"]);
    }
}
