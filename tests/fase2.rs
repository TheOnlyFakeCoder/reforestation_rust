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

/// Grafo y tabla w·h de 1 ha con la configuración por defecto (658 posiciones, sin vacías).
fn instancia_1ha() -> (Grafo, TablaCompetencia, usize) {
    use reforestation::competencia::{matriz_w, radio_max, radios};
    use reforestation::config::Config;
    use reforestation::datos::Datos;
    let config = Config::default();
    let especies = Datos::base().especies;
    let r = radios(&especies, config.d, config.factor_radios);
    let malla = Malla::para_objetivo(658, config.d);
    let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
    let w = matriz_w(&especies, config.kappa);
    let tabla = TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia);
    (Grafo::desde(&vecindad), tabla, malla.len())
}

/// φ inicial reproducible: la composición nominal de 1 ha (δ_j) barajada con una semilla.
fn phi_nominal_barajado(semilla: u64) -> Vec<u8> {
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let delta = [42, 196, 42, 42, 49, 38, 73, 64, 86, 26];
    let mut phi: Vec<u8> = (0..10u8)
        .flat_map(|j| std::iter::repeat_n(j + 1, delta[j as usize]))
        .collect();
    phi.shuffle(&mut rand_chacha::ChaCha8Rng::seed_from_u64(semilla));
    phi
}

#[test]
fn buscar_sin_iteraciones_devuelve_el_inicial_con_su_objetivo() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 2.0,
        theta: [1.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 0,
        max_sin_mejora: 100,
        prob_intercambio: 1.0,
        semilla: 1,
    };
    let inicial = phi_nominal_barajado(3);
    let resultado = buscar(&problema, inicial.clone(), &params);
    let z = z_total(&inicial, &grafo, &tabla);
    assert_eq!(resultado.phi, inicial);
    assert!((resultado.z - z).abs() < 1e-9);
    // γ·Z + Σ θ_j·D_j con θ = 1 y D_tot = 658.
    assert!((resultado.objetivo - (2.0 * z + 658.0)).abs() < 1e-9);
    assert_eq!(resultado.iteraciones, 0);
}

fn conteos(phi: &[u8]) -> [usize; 11] {
    let mut c = [0usize; 11];
    for &x in phi {
        c[x as usize] += 1;
    }
    c
}

#[test]
fn intercambios_mejoran_z_y_conservan_la_composicion() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 2_000,
        max_sin_mejora: 2_000,
        prob_intercambio: 1.0,
        semilla: 1,
    };
    let inicial = phi_nominal_barajado(3);
    let resultado = buscar(&problema, inicial.clone(), &params);
    assert!(resultado.iteraciones > 0);
    assert!(resultado.objetivo < problema.objetivo(&inicial) - 1.0);
    assert!((resultado.objetivo - problema.objetivo(&resultado.phi)).abs() < 1e-9);
    assert_eq!(conteos(&resultado.phi), conteos(&inicial));
}

#[test]
fn las_posiciones_fijas_nunca_cambian() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let fija: Vec<bool> = (0..n).map(|u| u % 5 == 0).collect();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: fija.clone(),
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 500,
        max_sin_mejora: 500,
        prob_intercambio: 1.0,
        semilla: 1,
    };
    let inicial = phi_nominal_barajado(5);
    let resultado = buscar(&problema, inicial.clone(), &params);
    assert!(resultado.objetivo < problema.objetivo(&inicial));
    for u in (0..n).filter(|&u| fija[u]) {
        assert_eq!(resultado.phi[u], inicial[u], "la posición fija {u} cambió");
    }
}

#[test]
fn con_gamma_cero_la_banda_lleva_al_optimo_lineal_de_mortalidad() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    // θ_j = c_j·(1 − s1_j); L = ceil(0.9·δ), U = floor(1.1·δ) para 1 ha (calculados a mano).
    let theta = [1.469, 1.0738, 4.563, 1.2558, 1.3104, 2.1294, 0.7632, 2.052, 4.7329, 3.0082];
    let l = vec![38, 177, 38, 38, 45, 35, 66, 58, 78, 24];
    let u = vec![46, 215, 46, 46, 53, 41, 80, 70, 94, 28];
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 0.0,
        theta,
        restriccion: Restriccion::Banda { l: l.clone(), u: u.clone() },
    };
    let params = ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 3_000,
        max_sin_mejora: 3_000,
        prob_intercambio: 0.0,
        semilla: 1,
    };
    let resultado = buscar(&problema, phi_nominal_barajado(7), &params);
    let d = conteos(&resultado.phi);
    // Óptimo de min Σθ·D con L ≤ D ≤ U y ΣD = 658 (verificado con un PLE en scipy).
    assert!(
        d[1..] == [38, 215, 38, 46, 46, 35, 80, 58, 78, 24],
        "con γ = 0 y Restriccion::Banda, buscar() debe reasignar especies (a → b dentro de \
         [L, U]) hasta el óptimo lineal [38, 215, 38, 46, 46, 35, 80, 58, 78, 24]; quedó {:?} \
         (la composición no cambió: falta el movimiento de reasignación)",
        &d[1..]
    );
}

#[test]
fn la_semilla_fija_el_resultado_y_otra_semilla_explora_distinto() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = |semilla| ParamsTabu {
        candidatos: 50,
        tenure: 10,
        max_iter: 300,
        max_sin_mejora: 300,
        prob_intercambio: 1.0,
        semilla,
    };
    let inicial = phi_nominal_barajado(9);
    let a = buscar(&problema, inicial.clone(), &params(1));
    let b = buscar(&problema, inicial.clone(), &params(1));
    let c = buscar(&problema, inicial, &params(2));
    assert!(a.phi == b.phi, "misma semilla, mismo resultado");
    assert!(
        a.phi != c.phi,
        "con semilla 2 el resultado es idéntico al de semilla 1: buscar() no usa la semilla \
         (los candidatos deben muestrearse al azar con ParamsTabu::semilla)"
    );
}

#[test]
fn se_detiene_tras_max_sin_mejora_iteraciones_sin_mejorar() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    // γ = 0 y composición fija: ningún intercambio cambia el objetivo, nunca hay mejora.
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 0.0,
        theta: [1.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 10,
        tenure: 10,
        max_iter: 5_000,
        max_sin_mejora: 25,
        prob_intercambio: 1.0,
        semilla: 1,
    };
    let resultado = buscar(&problema, phi_nominal_barajado(1), &params);
    assert!(
        resultado.iteraciones == 25,
        "sin ninguna mejora debe parar a las max_sin_mejora = 25 iteraciones, no a las {}",
        resultado.iteraciones
    );
}

#[test]
fn aplica_el_mejor_movimiento_aunque_empeore_y_devuelve_el_mejor_visto() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 20,
        tenure: 10,
        max_iter: 3_000,
        max_sin_mejora: 3_000,
        prob_intercambio: 1.0,
        semilla: 4,
    };
    let resultado = buscar(&problema, phi_nominal_barajado(2), &params);
    let traza = &resultado.traza;
    assert!(traza.len() == resultado.iteraciones, "una entrada de la traza por iteración");
    let empeoro = traza.windows(2).any(|par| par[1] > par[0] + 1e-12);
    assert!(empeoro, "la búsqueda tabú debe aceptar movimientos que empeoran (traza nunca sube)");
    let minimo = traza.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        (resultado.objetivo - minimo).abs() < 1e-6,
        "debe devolver la mejor solución vista: objetivo {} contra mínimo de la traza {minimo}",
        resultado.objetivo
    );
}

/// Malla de 6 posiciones (2 × 3, completa, d = 1) con dos especies, como en la prueba de Z.
fn instancia_chica() -> (Grafo, TablaCompetencia) {
    let malla = Malla::rectangular(2, 3, Disposicion::Completa, 1.0);
    let vecindad = malla.vecindad(2.5, 1e-9);
    let w = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
    let tabla = TablaCompetencia::nueva(&w, &[1.0, 2.5], &vecindad.anillos, 1.0, 1e-9);
    (Grafo::desde(&vecindad), tabla)
}

#[test]
fn la_lista_tabu_impide_deshacer_el_ultimo_movimiento() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla) = instancia_chica();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; 6],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    // 400 candidatos sobre 6 posiciones: en cada iteración se ven todos los intercambios.
    let traza = |tenure| {
        let params = ParamsTabu {
            candidatos: 400,
            tenure,
            max_iter: 60,
            max_sin_mejora: 60,
            prob_intercambio: 1.0,
            semilla: 3,
        };
        buscar(&problema, vec![1, 1, 1, 2, 2, 2], &params).traza
    };
    let distintos = |t: &[f64]| {
        let mut v = t.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v.len()
    };
    let sin_tabu = traza(0);
    let con_tabu = traza(3);
    assert!(
        distintos(&sin_tabu[20..]) <= 2,
        "sin lista tabú (tenure 0) la búsqueda debe ciclar entre dos arreglos"
    );
    assert!(
        distintos(&con_tabu[20..]) >= 3,
        "con tenure 3 una posición no puede volver a su especie anterior en 3 iteraciones, \
         así que la búsqueda no puede ciclar entre dos arreglos (buscar() ignora `tenure`)"
    );
}

#[test]
fn la_lista_tabu_tambien_aplica_a_las_reasignaciones() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla) = instancia_chica();
    let mut l = vec![0; 10];
    let mut u = vec![0; 10];
    (l[0], u[0], l[1], u[1]) = (2, 4, 2, 4);
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; 6],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::Banda { l, u },
    };
    // Sólo reasignaciones (prob_intercambio = 0) y todas vistas en cada iteración.
    let traza = |tenure| {
        let params = ParamsTabu {
            candidatos: 400,
            tenure,
            max_iter: 60,
            max_sin_mejora: 60,
            prob_intercambio: 0.0,
            semilla: 3,
        };
        buscar(&problema, vec![1, 1, 1, 2, 2, 2], &params).traza
    };
    let distintos = |t: &[f64]| {
        let mut v = t.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v.len()
    };
    assert!(
        distintos(&traza(0)[20..]) <= 2,
        "sin lista tabú las reasignaciones deben ciclar entre dos arreglos"
    );
    assert!(
        distintos(&traza(3)[20..]) >= 3,
        "con tenure 3 una posición reasignada no puede volver a su especie anterior en 3 \
         iteraciones (las reasignaciones ignoran la lista tabú)"
    );
}

#[test]
fn reporta_tiempo_y_curva_del_mejor_por_iteracion() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta: [0.0; 10],
        restriccion: Restriccion::ComposicionFija,
    };
    let params = ParamsTabu {
        candidatos: 20,
        tenure: 10,
        max_iter: 500,
        max_sin_mejora: 500,
        prob_intercambio: 1.0,
        semilla: 4,
    };
    let resultado = buscar(&problema, phi_nominal_barajado(2), &params);
    assert!(resultado.tiempo_s > 0.0, "debe medir el tiempo de ejecución");
    let curva = &resultado.curva;
    assert!(curva.len() == resultado.iteraciones, "una entrada de la curva por iteración");
    assert!(
        curva.windows(2).all(|par| par[1] <= par[0]),
        "la curva del mejor objetivo nunca sube"
    );
    assert!((curva[curva.len() - 1] - resultado.objetivo).abs() < 1e-6);
}

#[test]
fn la_banda_nunca_se_viola_en_una_busqueda_mixta() {
    use reforestation::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
    let (grafo, tabla, n) = instancia_1ha();
    let theta = [1.469, 1.0738, 4.563, 1.2558, 1.3104, 2.1294, 0.7632, 2.052, 4.7329, 3.0082];
    let l = vec![38, 177, 38, 38, 45, 35, 66, 58, 78, 24];
    let u = vec![46, 215, 46, 46, 53, 41, 80, 70, 94, 28];
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta,
        restriccion: Restriccion::Banda { l: l.clone(), u: u.clone() },
    };
    for semilla in 1..=5 {
        let params = ParamsTabu {
            candidatos: 30,
            tenure: 10,
            max_iter: 2_000,
            max_sin_mejora: 2_000,
            prob_intercambio: 0.5,
            semilla,
        };
        let resultado = buscar(&problema, phi_nominal_barajado(semilla), &params);
        let d = conteos(&resultado.phi);
        assert_eq!(d.iter().sum::<usize>(), 658);
        for j in 0..10 {
            assert!(
                (l[j]..=u[j]).contains(&d[j + 1]),
                "semilla {semilla}: especie {j} con {} fuera de [{}, {}]",
                d[j + 1],
                l[j],
                u[j]
            );
        }
    }
}
