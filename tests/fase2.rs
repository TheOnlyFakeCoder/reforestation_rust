//! Prueba de fase 2: objetivo Z, búsqueda tabú y paso 1.

use reforestation::competencia::TablaCompetencia;
use reforestation::malla::{Disposicion, Malla};
use reforestation::objetivo::{z_total, Grafo};

#[test]
fn z_total_de_malla_chica_calculado_a_mano() {
    // Posiciones A(0.5, ·) B(1.5, ·) / C(1, ·) D(2, ·) / E(0.5, ·) F(1.5, ·) con d = 1.
    // φ = A1 B2 C1 D2 E0 F1; especie 1 con R = d, especie 2 con R = 2.5d, w12 = 0.5.
    // Inmediatos: AB .5, CD .5, AC 1, BC .5, BD 1, CF 1, DF .5 (EF, CE vacías) = 5.
    // A √3: AD y BF son pares 1–2 con h = 1 − (√3 − 1)/1.5; AE y DE vacías. A 2d: AF tiene h = 0.
    let malla = Malla::rectangular(2, 3, Disposicion::Completa, 1.0);
    let vecindad = malla.vecindad(2.5, 1e-9);
    let w = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
    let tabla = TablaCompetencia::nueva(&w, &[1.0, 2.5], &vecindad.anillos, 1.0, 1e-9);
    let grafo = Grafo::desde(&vecindad);
    let phi = [1, 2, 1, 2, 0, 1];
    let esperado = 5.0 + 2.0 * 0.5 * (1.0 - (3f64.sqrt() - 1.0) / 1.5);
    assert!((z_total(&phi, &grafo, &tabla) - esperado).abs() < 1e-12);
}

#[test]
fn delta_coincide_con_recalculo_en_10000_movimientos() {
    use rand::{Rng, SeedableRng};
    use reforestation::competencia::{matriz_w, radio_max, radios};
    use reforestation::config::Config;
    use reforestation::datos::Datos;
    use reforestation::objetivo::{delta_intercambio, delta_reasignacion};

    let config = Config::default();
    let especies = Datos::base().especies;
    let r = radios(&especies, config.d, config.factor_radios);
    let malla = Malla::para_objetivo(658, config.d);
    let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
    let w = matriz_w(&especies, config.kappa);
    let tabla = TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia);
    let grafo = Grafo::desde(&vecindad);

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
    let n = malla.len();
    let mut phi: Vec<u8> = (0..n).map(|_| rng.gen_range(1..=10)).collect();
    for k in 0..40 {
        phi[k * 7 % n] = 0; // vacías
    }
    let fija: Vec<bool> = (0..n).map(|u| u % 6 == 1).collect(); // existentes, nunca se mueven
    let libres: Vec<usize> = (0..n).filter(|&u| !fija[u]).collect();

    let mut z = z_total(&phi, &grafo, &tabla);
    let (mut reasignaciones, mut intercambios, mut adyacentes) = (0, 0, 0);
    for k in 0..10_000 {
        let u = libres[rng.gen_range(0..libres.len())];
        let delta = if k % 2 == 0 {
            let b = rng.gen_range(0..=10);
            let delta = delta_reasignacion(&phi, u, b, &grafo, &tabla);
            phi[u] = b;
            reasignaciones += 1;
            delta
        } else {
            // Uno de cada cinco intercambios es entre vecinos, para cubrir el par (u, v) mismo.
            let v = if k % 10 == 1 {
                match grafo.vecinos(u).iter().map(|x| x.pos as usize).find(|&v| !fija[v]) {
                    Some(v) => {
                        adyacentes += 1;
                        v
                    }
                    None => continue,
                }
            } else {
                libres[rng.gen_range(0..libres.len())]
            };
            let delta = delta_intercambio(&phi, u, v, &grafo, &tabla);
            phi.swap(u, v);
            intercambios += 1;
            delta
        };
        let nuevo = z_total(&phi, &grafo, &tabla);
        assert!(
            (delta - (nuevo - z)).abs() < 1e-9,
            "movimiento {k}: delta {delta} contra recálculo {}",
            nuevo - z
        );
        z = nuevo;
    }
    assert!(reasignaciones >= 4_900 && intercambios >= 4_900 && adyacentes >= 900);
}
