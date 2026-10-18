//! Pruebas de fase 3: escenarios Monte Carlo.

use reforestation::escenarios::{estadisticos, generar, percentil, ParamsMc};

fn params() -> ParamsMc {
    ParamsMc { mu: 129.0, sigma: 3.3, p_hat: [0.1; 10] }
}

#[test]
fn misma_semilla_mismo_escenario() {
    let a = generar(658, 1.0, 0, 5, 42, &params());
    let b = generar(658, 1.0, 0, 5, 42, &params());
    assert_eq!(a, b);
    let c = generar(658, 1.0, 0, 6, 42, &params());
    assert_ne!(a.posiciones, c.posiciones);
    let d = generar(658, 1.0, 27, 5, 42, &params());
    assert_ne!(a.posiciones, d.posiciones);
}

#[test]
fn suma_x_es_t_y_posiciones_validas() {
    for omega in 0..50 {
        let e = generar(842, 1.28, 27, omega, 7, &params());
        assert_eq!(e.x.iter().sum::<u32>() as usize, e.t);
        assert_eq!(e.posiciones.len(), e.t);
        let mut vistas = std::collections::HashSet::new();
        let mut por_especie = [0u32; 10];
        for &(pos, esp) in &e.posiciones {
            assert!(pos < 842);
            assert!(vistas.insert(pos), "posición repetida");
            por_especie[esp as usize] += 1;
        }
        assert_eq!(por_especie, e.x);
        assert_eq!(e.t, (e.rho * 1.28).round() as usize);
    }
}

#[test]
fn t_se_acota_por_el_numero_de_posiciones() {
    let p = ParamsMc { mu: 1000.0, sigma: 1.0, p_hat: [0.1; 10] };
    let e = generar(50, 1.0, 1, 0, 1, &p);
    assert_eq!(e.t, 50);
}

#[test]
fn x_respeta_las_proporciones() {
    let mut p_hat = [0.0; 10];
    p_hat[3] = 1.0;
    let p = ParamsMc { mu: 100.0, sigma: 1.0, p_hat };
    let e = generar(658, 1.0, 0, 0, 1, &p);
    assert_eq!(e.x[3] as usize, e.t);
}

#[test]
fn percentil_interpola_como_numpy() {
    let v = [1.0, 2.0, 3.0, 4.0];
    assert!((percentil(&v, 0.5) - 2.5).abs() < 1e-12);
    assert!((percentil(&v, 0.9) - 3.7).abs() < 1e-12);
    assert_eq!(percentil(&v, 1.0), 4.0);
}

#[test]
fn estadisticos_de_una_muestra_conocida() {
    let (media, sd) = estadisticos(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    assert!((media - 5.0).abs() < 1e-12);
    assert!((sd - (32.0f64 / 7.0).sqrt()).abs() < 1e-12);
}

#[test]
fn resumen_de_escenarios_construidos_a_mano() {
    use reforestation::escenarios::{resumir, Escenario};
    let esc = |omega, x0: u32| Escenario {
        omega,
        rho: 0.0,
        t: x0 as usize,
        x: {
            let mut x = [0u32; 10];
            x[0] = x0;
            x
        },
        posiciones: vec![],
    };
    // D_0 = 10, L_0 = 11; X_0 = 8, 10, 12, 14 -> C_0 = 2, 0, -2, -4
    let escenarios = [esc(0, 8), esc(1, 10), esc(2, 12), esc(3, 14)];
    let mut d = [0u32; 10];
    d[0] = 10;
    let mut l = [100u32; 10];
    l[0] = 11;
    let mut costos = [0.0; 10];
    costos[0] = 2.0;
    let (especies, costo) = resumir(&escenarios, &d, &l, &costos, 3.0);
    assert!((especies[0].e_c - (-1.0)).abs() < 1e-12);
    assert!((especies[0].sd_c - (20.0f64 / 3.0).sqrt()).abs() < 1e-12);
    // costos 4, 0, 0, 0 -> media 1; supera B = 3 una de cuatro veces
    assert!((costo.costo_esperado - 1.0).abs() < 1e-12);
    assert_eq!(costo.p_costo_mayor_b, 0.25);
    assert_eq!(costo.violaciones_x_mayor_l, 2);
    assert_eq!((costo.escenarios_c_negativo, costo.especies_c_negativo), (2, 1));
}
