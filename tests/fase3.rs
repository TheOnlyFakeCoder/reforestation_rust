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

// ---------- Restriccion::Cuotas ----------

use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
use reforestation::competencia::{matriz_w, radio_max, radios, TablaCompetencia};
use reforestation::config::Config;
use reforestation::datos::Datos;
use reforestation::malla::Malla;
use reforestation::objetivo::{z_total, Grafo};

fn instancia_1ha() -> (Grafo, TablaCompetencia, usize) {
    let config = Config::default();
    let especies = Datos::base().especies;
    let r = radios(&especies, config.d, config.factor_radios);
    let malla = Malla::para_objetivo(658, config.d);
    let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
    let w = matriz_w(&especies, config.kappa);
    let tabla = TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia);
    (Grafo::desde(&vecindad), tabla, malla.len())
}

#[test]
fn cuotas_respeta_fijas_cuotas_y_total_y_el_delta_es_coherente() {
    let (grafo, tabla, n) = instancia_1ha();
    // Primeras 100 posiciones fijas (existentes), el resto libre.
    let mut fija = vec![false; n];
    let mut phi = vec![0u8; n];
    for u in 0..100 {
        fija[u] = true;
        phi[u] = (u % 10) as u8 + 1;
    }
    // 300 libres plantadas con la especie (u % 10) + 1 a partir de la posición 100.
    let cuotas = vec![40usize, 30, 30, 30, 30, 30, 30, 30, 30, 20];
    let mut restantes = cuotas.clone();
    let mut u = 100;
    'fuera: loop {
        for j in 0..10 {
            if restantes[j] > 0 {
                phi[u] = j as u8 + 1;
                restantes[j] -= 1;
                u += 1;
                if u >= n {
                    break 'fuera;
                }
            }
        }
        if restantes.iter().all(|&r| r == 0) {
            break;
        }
    }
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: fija.clone(),
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::Cuotas { n: cuotas.clone() },
    };
    let params = ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 1_500,
        max_sin_mejora: 1_500,
        prob_intercambio: 0.8,
        semilla: 3,
    };
    let libres_antes = (0..n).filter(|&u| !fija[u] && phi[u] > 0).count();
    let r = buscar(&problema, phi.clone(), &params);
    for u in 0..100 {
        assert_eq!(r.phi[u], phi[u], "existente movida");
    }
    let mut cuenta = [0usize; 11];
    for u in 0..n {
        if !fija[u] {
            cuenta[r.phi[u] as usize] += 1;
        }
    }
    for j in 0..10 {
        assert!(cuenta[j + 1] <= cuotas[j], "cuota {j} excedida");
    }
    assert_eq!(cuenta[1..].iter().sum::<usize>(), libres_antes);
    assert!((r.z - z_total(&r.phi, &grafo, &tabla)).abs() < 1e-9);
    assert!((r.objetivo - problema.objetivo(&r.phi)).abs() < 1e-9);
    assert!(r.objetivo < problema.objetivo(&phi) - 1.0, "no mejora");
}
