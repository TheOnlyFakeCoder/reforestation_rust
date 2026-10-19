//! Pruebas de fase 4: calibración del tabú.

use reforestation::calibracion::{calibrar, elegir, FilaCalibracion};
use reforestation::config::{Config, ConfigTabu};
use reforestation::datos::Datos;
use reforestation::paso1::Instancia;

fn fila(inst: &str, k: usize, t: usize, z: f64, tiempo: f64) -> FilaCalibracion {
    FilaCalibracion {
        instancia: inst.to_string(),
        candidatos: k,
        tenure: t,
        n_semillas: 5,
        z_opt_media: z,
        z_opt_sd: 0.0,
        z_opt_mejor: z,
        objetivo_media: 0.0,
        tiempo_medio_s: tiempo,
        iter_media: 0.0,
        frac_sin_mejora: 1.0,
    }
}

#[test]
fn elegir_gana_menor_z_rel_promedio() {
    let filas = vec![
        fila("a", 32, 10, 100.0, 1.0),
        fila("a", 64, 15, 110.0, 5.0),
        fila("b", 32, 10, 200.0, 1.0),
        fila("b", 64, 15, 200.0, 5.0),
    ];
    assert_eq!(elegir(&filas), (32, 10));
    let filas = vec![
        fila("a", 32, 10, 110.0, 1.0),
        fila("a", 64, 15, 100.0, 5.0),
        fila("b", 32, 10, 210.0, 1.0),
        fila("b", 64, 15, 200.0, 5.0),
    ];
    assert_eq!(elegir(&filas), (64, 15));
}

#[test]
fn elegir_empate_menor_a_una_milesima_toma_la_mas_rapida() {
    let filas = vec![
        fila("a", 32, 10, 100.0, 9.0),
        fila("a", 64, 15, 100.05, 2.0),
        fila("a", 128, 25, 101.0, 0.5),
    ];
    // 64/15 queda a 0.0005 del mejor y es más rápida; 128/25 queda fuera.
    assert_eq!(elegir(&filas), (64, 15));
}

#[test]
fn calibrar_rejilla_2x1_devuelve_dos_filas() {
    let datos = Datos::base();
    let config = Config::default();
    let inst = Instancia::nueva("1ha", 1.0, &datos, &config, config.m_escala, config.usar_s2);
    let base = ConfigTabu {
        candidatos: 20,
        tenure: 5,
        max_iter: 200,
        max_sin_mejora: 200,
        prob_intercambio: 0.5,
        iter_por_posicion: 0,
        sin_mejora_por_posicion: 0,
    };
    let filas = calibrar(&inst, 3.0, &base, &[16, 24], &[5], &[1, 2]);
    assert_eq!(filas.len(), 2);
    for f in &filas {
        assert_eq!(f.instancia, "1ha");
        assert_eq!(f.n_semillas, 2);
        assert!(f.z_opt_sd >= 0.0);
        assert!(f.z_opt_mejor <= f.z_opt_media + 1e-9);
        assert!((0.0..=1.0).contains(&f.frac_sin_mejora));
    }
    assert_eq!((filas[0].candidatos, filas[1].candidatos), (16, 24));
}
