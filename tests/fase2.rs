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
                match grafo
                    .vecinos(u)
                    .iter()
                    .map(|x| x.pos as usize)
                    .find(|&v| !fija[v])
                {
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
    let theta = [
        1.469, 1.0738, 4.563, 1.2558, 1.3104, 2.1294, 0.7632, 2.052, 4.7329, 3.0082,
    ];
    let l = vec![38, 177, 38, 38, 45, 35, 66, 58, 78, 24];
    let u = vec![46, 215, 46, 46, 53, 41, 80, 70, 94, 28];
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 0.0,
        theta,
        restriccion: Restriccion::Banda {
            l: l.clone(),
            u: u.clone(),
        },
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
    assert!(
        traza.len() == resultado.iteraciones,
        "una entrada de la traza por iteración"
    );
    let empeoro = traza.windows(2).any(|par| par[1] > par[0] + 1e-12);
    assert!(
        empeoro,
        "la búsqueda tabú debe aceptar movimientos que empeoran (traza nunca sube)"
    );
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
    assert!(
        resultado.tiempo_s > 0.0,
        "debe medir el tiempo de ejecución"
    );
    let curva = &resultado.curva;
    assert!(
        curva.len() == resultado.iteraciones,
        "una entrada de la curva por iteración"
    );
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
    let theta = [
        1.469, 1.0738, 4.563, 1.2558, 1.3104, 2.1294, 0.7632, 2.052, 4.7329, 3.0082,
    ];
    let l = vec![38, 177, 38, 38, 45, 35, 66, 58, 78, 24];
    let u = vec![46, 215, 46, 46, 53, 41, 80, 70, 94, 28];
    let problema = Problema {
        grafo: &grafo,
        tabla: &tabla,
        fija: vec![false; n],
        gamma: 1.0,
        theta,
        restriccion: Restriccion::Banda {
            l: l.clone(),
            u: u.clone(),
        },
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

#[test]
fn theta_es_costo_por_mortalidad_con_escala_y_s2() {
    use reforestation::datos::Datos;
    use reforestation::paso1::theta;
    let especies = Datos::base().especies;
    let t = theta(&especies, 1.0, false);
    // θ_j = c_j·(1 − s1_j), calculado a mano.
    assert!((t[0] - 26.0 * (1.0 - 0.9435)).abs() < 1e-12);
    assert!((t[8] - 26.5 * (1.0 - 0.8214)).abs() < 1e-12);
    // m_j = 0.5·c_j y s2.
    let t = theta(&especies, 0.5, true);
    assert!((t[9] - 0.5 * 26.0 * (1.0 - 0.7263)).abs() < 1e-12);
}

#[test]
fn serpentina_invierte_el_sentido_en_filas_impares_y_deja_vacio_el_excedente() {
    use reforestation::paso1::patron_serpentina;
    // Filas de 3: fila 0 = posiciones 0, 1, 2 (izquierda → derecha); fila 1 = 3, 4, 5.
    // Recorrido: 0, 1, 2, 5, 4, 3. Secuencia [0, 1] → códigos 1, 2, 1, 2, 1 y la última vacía.
    let malla = Malla::rectangular(3, 2, Disposicion::Completa, 1.0);
    assert_eq!(
        patron_serpentina(&malla, &[0, 1], 5),
        vec![1, 2, 1, 0, 1, 2]
    );
}

#[test]
fn reparar_a_banda_deja_la_serpentina_dentro_de_l_u_sin_tocar_las_vacias() {
    use reforestation::composicion::Banda;
    use reforestation::paso1::{patron_serpentina, reparar_a_banda};
    use reforestation::tabu::DEFAULT_ROW_SEQUENCE;
    let malla = Malla::para_objetivo(658, 4.189171180993856);
    let mut phi = patron_serpentina(&malla, DEFAULT_ROW_SEQUENCE, 658);
    // Con la secuencia de 21 plantas, lechuguilla (idx 0) queda en 31 < L = 38.
    assert_eq!(conteos(&phi)[1], 31);
    phi[10] = 0;
    phi[400] = 0;
    let banda = Banda {
        l: vec![38, 177, 38, 38, 45, 35, 66, 58, 78, 24],
        u: vec![46, 215, 46, 46, 53, 41, 80, 70, 94, 28],
    };
    reparar_a_banda(&mut phi, &banda);
    let d = conteos(&phi);
    assert_eq!(d[0], 2, "las vacías no cambian");
    assert!(phi[10] == 0 && phi[400] == 0);
    for j in 0..10 {
        assert!(
            (banda.l[j]..=banda.u[j]).contains(&d[j + 1]),
            "especie {j} con {} fuera de [{}, {}]",
            d[j + 1],
            banda.l[j],
            banda.u[j]
        );
    }
}

fn params_cortos(semilla: u64) -> reforestation::busqueda_tabu::ParamsTabu {
    reforestation::busqueda_tabu::ParamsTabu {
        candidatos: 30,
        tenure: 10,
        max_iter: 1_500,
        max_sin_mejora: 1_500,
        prob_intercambio: 0.5,
        semilla,
    }
}

#[test]
fn resolver_paso1_reoptimiza_z_con_la_composicion_fija() {
    use reforestation::config::Config;
    use reforestation::datos::Datos;
    use reforestation::paso1::{resolver_paso1, Instancia};
    let instancia = Instancia::nueva("1ha", 1.0, &Datos::base(), &Config::default(), 1.0, false);
    let (fila, phi) = resolver_paso1(&instancia, 1.0, &params_cortos(1));
    assert!(
        fila.z_opt <= fila.z_paso1 + 1e-9,
        "el ajuste C (sólo intercambios) no puede empeorar Z: {} > {}",
        fila.z_opt,
        fila.z_paso1
    );
    let d = conteos(&phi);
    assert!(
        d[1..] == fila.d,
        "el patrón devuelto tiene la composición D*"
    );
    assert!(
        phi.len() == 658,
        "φ* debe cubrir las 658 posiciones de 1 ha"
    );
    assert!(
        fila.d.iter().sum::<usize>() == 658,
        "Σ D* debe ser D_tot = 658 (φ* debe plantar todas las posiciones de 1 ha)"
    );
    let (l, u) = (
        [38, 177, 38, 38, 45, 35, 66, 58, 78, 24],
        [46, 215, 46, 46, 53, 41, 80, 70, 94, 28],
    );
    assert!(
        (0..10).all(|j| l[j] <= fila.d[j] && fila.d[j] <= u[j]),
        "D* (los conteos de φ*) debe quedar en la banda [L, U] de 1 ha y quedó {:?}: φ* es la \
         serpentina de la socia sin pasar por reparar_a_banda(φ, banda de la instancia)",
        fila.d
    );
    let z = z_total(&phi, &instancia.grafo, &instancia.tabla);
    assert!(
        (fila.z_opt - z).abs() < 1e-9,
        "z_opt debe ser la Z de φ* ({z}) y vale {}",
        fila.z_opt
    );
}

#[test]
fn resolver_paso1_mejora_la_serpentina_de_la_socia() {
    use reforestation::config::Config;
    use reforestation::datos::Datos;
    use reforestation::paso1::{patron_serpentina, reparar_a_banda, resolver_paso1, Instancia};
    use reforestation::tabu::DEFAULT_ROW_SEQUENCE;
    let instancia = Instancia::nueva("1ha", 1.0, &Datos::base(), &Config::default(), 1.0, false);
    let (fila, _) = resolver_paso1(&instancia, 1.0, &params_cortos(1));
    let mut socia = patron_serpentina(&instancia.malla, DEFAULT_ROW_SEQUENCE, 658);
    reparar_a_banda(&mut socia, &instancia.banda);
    let z_socia = z_total(&socia, &instancia.grafo, &instancia.tabla);
    assert!(
        fila.z_opt < z_socia - 1.0,
        "la búsqueda tabú (paso 1 + ajuste C) debe mejorar la serpentina reparada: z_opt {} \
         contra {z_socia} (resolver_paso1 no corre buscar())",
        fila.z_opt
    );
}

#[test]
fn barrido_con_2_gammas_y_2_semillas_produce_4_filas_con_d_en_banda() {
    use reforestation::config::Config;
    use reforestation::datos::Datos;
    use reforestation::paso1::{barrido_gamma, Instancia};
    let instancia = Instancia::nueva("1ha", 1.0, &Datos::base(), &Config::default(), 1.0, false);
    let filas = barrido_gamma(&instancia, &[0.0, 1.0], &[1, 2], &params_cortos(0));
    // 2 γ × 2 semillas = 4 filas; la regla de extensión puede agregar hasta 3 γ más (2 filas c/u).
    assert!(
        filas.len() >= 4 && filas.len() <= 10 && filas.len().is_multiple_of(2),
        "{}",
        filas.len()
    );
    let gammas: Vec<f64> = filas.iter().take(4).map(|f| f.gamma).collect();
    assert_eq!(gammas, vec![0.0, 0.0, 1.0, 1.0]);
    assert_eq!(
        filas.iter().map(|f| f.semilla).take(4).collect::<Vec<_>>(),
        vec![1, 2, 1, 2]
    );
    for f in &filas {
        assert_eq!(f.instancia, "1ha");
        assert_eq!(f.map_id, 0);
        assert_eq!(f.d_tot, 658);
        assert_eq!(f.d.iter().sum::<usize>(), 658);
        for j in 0..10 {
            assert!(instancia.banda.l[j] <= f.d[j] && f.d[j] <= instancia.banda.u[j]);
        }
        assert!((f.z_opt_por_planta - f.z_opt / 658.0).abs() < 1e-12);
        let theta: f64 = (0..10).map(|j| instancia.theta[j] * f.d[j] as f64).sum();
        let costo: f64 = (0..10).map(|j| instancia.costos[j] * f.d[j] as f64).sum();
        assert!((f.suma_theta_d - theta).abs() < 1e-9 && (f.suma_c_d - costo).abs() < 1e-9);
        assert!(f.iter_paso1 > 0 && f.iter_opt > 0);
    }
}

#[test]
fn barrido_agrega_gamma_por_3_mientras_d_cambia_en_el_extremo_hasta_3_veces() {
    use reforestation::paso1::{barrido_con, FilaPaso1};
    fn fila(gamma: f64, a: usize) -> FilaPaso1 {
        FilaPaso1 {
            gamma,
            d: [a; 10],
            ..FilaPaso1::vacia()
        }
    }
    // D cambia siempre: se agregan 100·3, 300·3 y 900·3 y se detiene (3 extensiones).
    let filas = barrido_con(&[1.0, 10.0, 100.0], &[7], |g, _| fila(g, g as usize));
    let gammas: Vec<f64> = filas.iter().map(|f| f.gamma).collect();
    assert_eq!(gammas, vec![1.0, 10.0, 100.0, 300.0, 900.0, 2700.0]);
    // D estable en los dos γ mayores: no se agrega nada.
    let filas = barrido_con(&[1.0, 10.0, 100.0], &[7], |g, _| {
        fila(g, if g < 5.0 { 1 } else { 2 })
    });
    assert_eq!(filas.len(), 3);
    // Cambia una vez y luego se estabiliza: una extensión.
    let filas = barrido_con(&[1.0, 100.0], &[7], |g, _| {
        fila(g, if g < 50.0 { 1 } else { 2 })
    });
    assert_eq!(
        filas.iter().map(|f| f.gamma).collect::<Vec<_>>(),
        vec![1.0, 100.0, 300.0]
    );
}
