//! Prueba de fase 1: datos, malla, composición y competencia trabajando juntos.
#![allow(clippy::needless_range_loop)]

use reforestation::competencia::{matriz_w, radio_max, radios, TablaCompetencia};
use reforestation::composicion::{banda, composicion_nominal, d_tot};
use reforestation::config::Config;
use reforestation::datos::Datos;
use reforestation::malla::Malla;

// Codificación φ: especie j ↔ j + 1.
const STREPTACANTHA: u8 = 8;
const PROSOPIS: u8 = 9;

/// Tabla w·h de 1 ha con un factor de radios dado, junto con el número de anillos y R_max/d.
fn tabla_1ha(factor: f64) -> (TablaCompetencia, usize, f64) {
    let config = Config::default();
    let especies = Datos::base().especies;
    let r = radios(&especies, config.d, factor);
    let radio = radio_max(&r, config.d);
    let vecindad = Malla::para_objetivo(658, config.d).vecindad(radio, config.tolerancia);
    let w = matriz_w(&especies, config.kappa);
    let tabla = TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia);
    (tabla, vecindad.anillos.len(), radio / config.d)
}

#[test]
fn factor_1_25_prosopis_compite_en_raiz7_d_y_3d() {
    let (tabla, n_anillos, radio) = tabla_1ha(1.25);
    assert!((radio - 3.125).abs() < 1e-9);
    assert_eq!(n_anillos, 5);
    // R = 3.125d: h(√7·d) = 1 − (√7 − 1)/2.125 = 0.22553; h(3d) = 1 − 2/2.125 = 0.05882.
    assert!((tabla.peso(3, PROSOPIS, PROSOPIS) - 0.22553).abs() < 1e-4);
    assert!((tabla.peso(4, PROSOPIS, PROSOPIS) - 0.05882).abs() < 1e-4);
    // O. streptacantha con R = 2.5d ya no alcanza √7·d.
    assert_eq!(tabla.peso(3, STREPTACANTHA, STREPTACANTHA), 0.0);
}

#[test]
fn factor_0_75_conserva_h_1_en_inmediatos_y_r_max_1_875d() {
    let (tabla, n_anillos, radio) = tabla_1ha(0.75);
    let w = matriz_w(&Datos::base().especies, Config::default().kappa);
    assert!((radio - 1.875).abs() < 1e-9);
    assert_eq!(n_anillos, 2);
    for j in 0..10 {
        for k in 0..10 {
            assert_eq!(tabla.peso(0, j as u8 + 1, k as u8 + 1), w[j][k], "par ({j}, {k})");
        }
    }
    // R = 1.875d: h(√3·d) = 1 − (√3 − 1)/0.875 = 0.16337; streptacantha (R = 1.5d) no llega.
    assert!((tabla.peso(1, PROSOPIS, PROSOPIS) - 0.16337).abs() < 1e-4);
    assert_eq!(tabla.peso(1, STREPTACANTHA, STREPTACANTHA), 0.0);
}

#[test]
fn los_30_poligonos_tienen_malla_y_composicion_consistentes() {
    let datos = Datos::base();
    let config = Config::default();
    let delta: Vec<u32> = datos.especies.iter().map(|e| e.delta).collect();
    let r = radios(&datos.especies, config.d, config.factor_radios);
    // 1 ha (map_id 0 en los mensajes) más los 30 polígonos.
    let instancias = std::iter::once((0, 1.0)).chain(datos.poligonos.iter().map(|p| (p.map_id, p.area)));
    for (map_id, area) in instancias {
        let objetivo = (658.0 * area).round() as usize;
        let malla = Malla::para_objetivo(objetivo, config.d);
        // Malla rectangular casi cuadrada: excedente máximo observado 8 (map 7, 4140 vs 4132).
        let excedente = malla.len() - objetivo;
        assert!(excedente <= 8, "map {map_id}: excedente {excedente}");
        let proporcion = malla.ancho / malla.alto;
        assert!((0.8..=1.25).contains(&proporcion), "map {map_id}: ancho/alto {proporcion}");

        let total = d_tot(malla.len(), area, datos.densidad_total());
        assert_eq!(total, objetivo, "map {map_id}: D_tot debe ser round(658·A)");
        let nominal = composicion_nominal(total, &delta);
        assert_eq!(nominal.iter().sum::<usize>(), total, "map {map_id}");
        let b = banda(&nominal, config.epsilon);
        for j in 0..nominal.len() {
            assert!(b.l[j] <= nominal[j] && nominal[j] <= b.u[j], "map {map_id} especie {j}");
        }

        let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
        assert_eq!(vecindad.anillos.len(), 3, "map {map_id}");
        assert_eq!(vecindad.max_vecinos(), 18, "map {map_id}");
    }
}
