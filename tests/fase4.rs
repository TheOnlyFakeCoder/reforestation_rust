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

// ---------------------------------------------------------------------------------------------
// Instancias de validación (Task 5).

use reforestation::busqueda_tabu::{Problema, Restriccion};
use reforestation::composicion::{banda, composicion_nominal};
use reforestation::escenarios::ParamsMc;
use reforestation::objetivo::z_total;
use reforestation::validacion::{
    construir, correr_tabu, evaluar, inicial_paso3, red, violaciones, Especificacion, InstanciaVal,
    Modelo, Red, ESPECIFICACIONES,
};

const MC_JSON: &str = r#"{"mu":129.18,"sigma":3.29,"p_hat":[0.063,0.295,0.062,0.060,0.078,0.062,0.117,0.097,0.128,0.038]}"#;

fn mc() -> ParamsMc {
    serde_json::from_str(MC_JSON).unwrap()
}

fn espec(id: &str) -> &'static Especificacion {
    ESPECIFICACIONES.iter().find(|e| e.id == id).unwrap()
}

fn inst(id: &str) -> (InstanciaVal, Red) {
    let (datos, config) = (Datos::base(), Config::default());
    let e = espec(id);
    (
        construir(e, &datos, &config, &mc()),
        red(e.columnas, e.filas, &datos, &config),
    )
}

fn conteo_fijas(i: &InstanciaVal) -> (usize, [usize; 10]) {
    let mut x = [0usize; 10];
    for &f in &i.fija {
        if f >= 0 {
            x[f as usize] += 1;
        }
    }
    (x.iter().sum(), x)
}

const TABU_VAL_CORTO: ConfigTabu = ConfigTabu {
    candidatos: 20,
    tenure: 5,
    max_iter: 400,
    max_sin_mejora: 150,
    prob_intercambio: 0.5,
    iter_por_posicion: 0,
    sin_mejora_por_posicion: 0,
};

fn deltas() -> Vec<u32> {
    Datos::base().especies.iter().map(|e| e.delta).collect()
}

#[test]
fn las_diez_instancias_tienen_los_ids_modelos_y_tamanos_del_brief() {
    let esperadas = [
        ("p3_a", 49),
        ("p3_b", 81),
        ("p3_c", 49),
        ("p3_d", 64),
        ("p3_e", 100),
        ("p3_f", 64),
        ("p3_g", 100),
        ("p1_a", 49),
        ("p1_b", 64),
        ("p1_c", 100),
    ];
    assert_eq!(ESPECIFICACIONES.len(), 10);
    for (e, (id, n)) in ESPECIFICACIONES.iter().zip(esperadas) {
        assert_eq!(e.id, id);
        assert_eq!(e.columnas * e.filas, n);
        let (i, _) = inst(id);
        assert_eq!(i.id, id);
        assert_eq!(i.n_pos, n);
        assert_eq!(i.posiciones.len(), n);
        assert_eq!(i.fija.len(), n);
        assert_eq!(i.disposicion, "Completa");
        let modelo = if id.starts_with("p1") {
            Modelo::Paso1
        } else {
            Modelo::Paso3
        };
        assert_eq!(i.modelo, modelo);
        assert_eq!(i.semilla_existentes, Config::default().semilla);
    }
}

#[test]
fn p3_c_tiene_49_posiciones_10_fijas_y_suma_n_cubre_m_igual_v_lib() {
    let (i, _) = inst("p3_c");
    assert_eq!(i.n_pos, 49);
    let (t, x) = conteo_fijas(&i);
    assert_eq!(t, 10, "round(0.2·49) = 10 existentes");
    let v_lib = 49 - t;
    let n = i.cuotas.as_ref().expect("paso 3 tiene cuotas");
    assert_eq!(i.m, Some(v_lib));
    assert!(n.iter().sum::<usize>() >= v_lib);
    // n = C⁺ = (D − X)⁺ con D la nominal de |V|.
    let d = composicion_nominal(49, &deltas());
    assert_eq!(i.d_nominal, d);
    for j in 0..10 {
        assert_eq!(n[j], d[j].saturating_sub(x[j]), "especie {j}");
    }
    assert_eq!(i.d_tot, None);
    assert_eq!(i.n_especies_libres, None);
    assert_eq!(i.gamma, 1.0);
    assert_eq!(i.theta, vec![0.0; 10]);
}

#[test]
fn p3_f_con_huecos_tiene_m_igual_suma_n_menor_que_v_lib() {
    let (i, _) = inst("p3_f");
    let (t, x) = conteo_fijas(&i);
    assert_eq!(t, 13, "round(0.2·64) = 13 existentes");
    let v_lib = 64 - t;
    let n = i.cuotas.as_ref().unwrap();
    let suma: usize = n.iter().sum();
    assert_eq!(i.m, Some(suma));
    assert!(suma < v_lib, "M = {suma} debe ser < |V_lib| = {v_lib}");
    for j in 0..10 {
        let c = i.d_nominal[j].saturating_sub(x[j]) as f64;
        assert_eq!(n[j], (0.9 * c).round() as usize, "especie {j}");
    }
    // p3_d es la misma malla con la misma semilla: mismas existentes.
    let (d, _) = inst("p3_d");
    assert_eq!(d.fija, i.fija);
}

#[test]
fn p3_a_sin_existentes_usa_n_igual_d_y_m_igual_v() {
    let (i, _) = inst("p3_a");
    assert_eq!(conteo_fijas(&i).0, 0);
    assert_eq!(i.cuotas.as_ref().unwrap(), &i.d_nominal);
    assert_eq!(i.m, Some(49));
    assert_eq!(i.objetivo_constante, 0.0);
}

#[test]
fn p1_tiene_d_tot_v_banda_del_config_y_especies_libres() {
    let (datos, config) = (Datos::base(), Config::default());
    let (i, _) = inst("p1_b");
    assert_eq!(i.d_tot, Some(64));
    assert_eq!(i.cuotas, None);
    assert_eq!(i.m, None);
    assert_eq!(conteo_fijas(&i).0, 0);
    let b = banda(&i.d_nominal, config.epsilon);
    assert_eq!(
        (i.banda_l.clone(), i.banda_u.clone()),
        (b.l.clone(), b.u.clone())
    );
    let libres = (0..10).filter(|&j| b.l[j] < b.u[j]).count();
    assert_eq!(i.n_especies_libres, Some(libres));
    assert_eq!(i.gamma, config.gamma_star);
    let theta = reforestation::paso1::theta(&datos.especies, config.m_escala, config.usar_s2);
    assert_eq!(i.theta, theta.to_vec());
}

#[test]
fn aristas_son_los_pares_por_anillo_de_la_malla_y_peso_es_la_tabla() {
    for id in ["p3_c", "p1_c"] {
        let (i, r) = inst(id);
        let pares = r.vecindad.pares_por_anillo();
        let mut cuenta = vec![0; pares.len()];
        for &(u, v, a) in &i.aristas {
            assert!(u < v);
            let delta = r.malla.distancia(u, v);
            assert!((delta - r.vecindad.anillos[a as usize]).abs() < 1e-6);
            cuenta[a as usize] += 1;
        }
        assert_eq!(cuenta, pares, "{id}");
        assert_eq!(i.peso.len(), r.tabla.n_anillos());
        for a in 0..i.peso.len() {
            for j in 0..10 {
                for k in 0..10 {
                    assert_eq!(
                        i.peso[a][j][k],
                        r.tabla.peso(a as u16, j as u8 + 1, k as u8 + 1)
                    );
                }
            }
        }
        for (u, p) in r.malla.posiciones.iter().enumerate() {
            assert_eq!(i.posiciones[u], [p.x, p.y]);
        }
    }
}

#[test]
fn evaluar_coincide_con_z_total_incluidos_pares_existente_existente() {
    let (i, r) = inst("p3_c");
    let phi = inicial_paso3(&i, &r.malla);
    let (z, objetivo) = evaluar(&i, &phi);
    assert_eq!(z, z_total(&phi, &r.grafo, &r.tabla));
    assert_eq!(objetivo, z);
    // Sólo las existentes: Z son los pares existente-existente = objetivo_constante.
    let solo: Vec<u8> = i.fija.iter().map(|&f| (f + 1) as u8).collect();
    let (z0, _) = evaluar(&i, &solo);
    assert_eq!(z0, z_total(&solo, &r.grafo, &r.tabla));
    assert_eq!(z0, i.objetivo_constante);
    assert!(z0 > 0.0, "hay pares existente-existente en p3_c");
}

#[test]
fn evaluar_con_vacias_no_les_da_peso() {
    let (i, r) = inst("p3_f");
    let phi = inicial_paso3(&i, &r.malla);
    assert!(phi.contains(&0), "p3_f deja libres vacías");
    let (z, _) = evaluar(&i, &phi);
    assert_eq!(z, z_total(&phi, &r.grafo, &r.tabla));
    let a_mano: f64 = i
        .aristas
        .iter()
        .filter(|&&(u, v, _)| phi[u] > 0 && phi[v] > 0)
        .map(|&(u, v, a)| i.peso[a as usize][phi[u] as usize - 1][phi[v] as usize - 1])
        .sum();
    assert!((z - a_mano).abs() < 1e-9);
}

#[test]
fn evaluar_paso1_es_gamma_z_mas_suma_theta_d() {
    let (i, r) = inst("p1_a");
    let phi: Vec<u8> = (0..49).map(|u| (u % 10) as u8 + 1).collect();
    let (z, objetivo) = evaluar(&i, &phi);
    let problema = Problema {
        grafo: &r.grafo,
        tabla: &r.tabla,
        fija: vec![false; 49],
        gamma: i.gamma,
        theta: std::array::from_fn(|j| i.theta[j]),
        restriccion: Restriccion::ComposicionFija,
    };
    assert_eq!(z, z_total(&phi, &r.grafo, &r.tabla));
    assert_eq!(objetivo, problema.objetivo(&phi));
}

#[test]
fn json_ida_y_vuelta_conserva_los_pesos_exactos() {
    for id in ["p3_g", "p1_c"] {
        let (i, _) = inst(id);
        let texto = serde_json::to_string(&i).unwrap();
        let vuelta: InstanciaVal = serde_json::from_str(&texto).unwrap();
        assert_eq!(vuelta, i);
        let valor: serde_json::Value = serde_json::from_str(&texto).unwrap();
        for campo in [
            "id",
            "modelo",
            "columnas",
            "filas",
            "disposicion",
            "n_pos",
            "posiciones",
            "aristas",
            "peso",
            "fija",
            "cuotas",
            "m",
            "banda_l",
            "banda_u",
            "n_especies_libres",
            "d_tot",
            "theta",
            "gamma",
            "semilla_existentes",
            "objetivo_constante",
        ] {
            assert!(valor.get(campo).is_some(), "falta {campo}");
        }
        let modelo = if id == "p1_c" { "paso1" } else { "paso3" };
        assert_eq!(valor["modelo"], modelo);
    }
}

#[test]
fn inicial_paso3_respeta_fijas_cuotas_y_m() {
    for id in ["p3_a", "p3_c", "p3_f", "p3_g"] {
        let (i, r) = inst(id);
        let phi = inicial_paso3(&i, &r.malla);
        assert_eq!(violaciones(&i, &phi), Vec::<String>::new(), "{id}");
    }
}

#[test]
fn violaciones_detecta_fija_movida_cuota_excedida_y_m() {
    let (i, r) = inst("p3_f");
    let phi = inicial_paso3(&i, &r.malla);
    let u_fija = i.fija.iter().position(|&f| f >= 0).unwrap();
    let mut movida = phi.clone();
    movida[u_fija] = if movida[u_fija] == 1 { 2 } else { 1 };
    assert!(!violaciones(&i, &movida).is_empty());
    // Plantar una libre vacía: M + 1 plantas.
    let mut de_mas = phi.clone();
    let u_vacia = (0..64).find(|&u| i.fija[u] < 0 && phi[u] == 0).unwrap();
    de_mas[u_vacia] = 1;
    assert!(!violaciones(&i, &de_mas).is_empty());
    // Paso 1: salir de la banda.
    let (p1, _) = inst("p1_a");
    assert!(!violaciones(&p1, &[1u8; 49]).is_empty());
}

#[test]
fn tabu_val_paso3_respeta_cuotas_fijas_y_m() {
    for id in ["p3_a", "p3_c", "p3_f"] {
        let (i, r) = inst(id);
        let inicial = evaluar(&i, &inicial_paso3(&i, &r.malla)).0;
        let c = correr_tabu(&i, &r, &TABU_VAL_CORTO, 7);
        assert_eq!(c.semilla, 7);
        assert_eq!(violaciones(&i, &c.phi), Vec::<String>::new(), "{id}");
        assert_eq!((c.z, c.objetivo), evaluar(&i, &c.phi));
        assert!(c.z <= inicial + 1e-9, "{id}: {} > {inicial}", c.z);
        assert!(c.iteraciones > 0);
    }
}

#[test]
fn tabu_val_paso1_respeta_la_banda_y_reporta_gamma_z_mas_theta_d() {
    let (i, r) = inst("p1_a");
    let c = correr_tabu(&i, &r, &TABU_VAL_CORTO, 3);
    assert_eq!(violaciones(&i, &c.phi), Vec::<String>::new());
    assert_eq!((c.z, c.objetivo), evaluar(&i, &c.phi));
    assert!(c.objetivo > c.z * i.gamma, "θ > 0 suma mortalidad");
}

fn dir_temporal(nombre: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("reto_val_{}_{}", nombre, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn reto(args: &[&str]) -> std::process::Output {
    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_reto"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    salida
}

#[test]
fn cli_instancias_val_tabu_val_y_evaluar_val() {
    let datos = dir_temporal("datos");
    let res = dir_temporal("res");
    let (d, r) = (datos.to_str().unwrap(), res.to_str().unwrap());
    reto(&["datos", "--datos", d]);
    std::fs::write(res.join("mc_params.json"), MC_JSON).unwrap();
    reto(&["instancias-val", "--datos", d, "--resultados", r]);
    let val = res.join("validacion");
    for e in ESPECIFICACIONES.iter() {
        let texto = std::fs::read_to_string(val.join(format!("inst_{}.json", e.id))).unwrap();
        let i: InstanciaVal = serde_json::from_str(&texto).unwrap();
        assert_eq!(i, construir(e, &Datos::base(), &Config::default(), &mc()));
    }
    let corto = r#"tabu={"candidatos":20,"tenure":5,"max_iter":300,"max_sin_mejora":100,"prob_intercambio":0.5,"iter_por_posicion":0,"sin_mejora_por_posicion":0}"#;
    reto(&[
        "tabu-val",
        "--datos",
        d,
        "--resultados",
        r,
        "--semillas",
        "2",
        "--set",
        corto,
    ]);
    let semilla = Config::default().semilla;
    for e in ESPECIFICACIONES.iter() {
        let texto = std::fs::read_to_string(val.join(format!("tabu_{}.csv", e.id))).unwrap();
        let mut lineas = texto.lines();
        assert_eq!(
            lineas.next().unwrap(),
            "semilla,z,objetivo,tiempo_s,iteraciones,motivo_paro,phi"
        );
        let filas: Vec<&str> = lineas.collect();
        assert_eq!(filas.len(), 2, "{}", e.id);
        for (k, fila) in filas.iter().enumerate() {
            let campos: Vec<&str> = fila.split(',').collect();
            assert_eq!(campos[0], (semilla + k as u64).to_string());
            assert!(["tope", "sin_mejora"].contains(&campos[5]));
            let phi: Vec<i32> = campos[6].split(';').map(|s| s.parse().unwrap()).collect();
            assert_eq!(phi.len(), e.columnas * e.filas);
            assert!(phi.iter().all(|&s| (-1..10).contains(&s)));
        }
    }
    // evaluar-val sobre el CSV del tabú reproduce z y objetivo de cada fila.
    let inst_f = val.join("inst_p1_a.json");
    let tabu_f = val.join("tabu_p1_a.csv");
    let salida = reto(&[
        "evaluar-val",
        "--inst",
        inst_f.to_str().unwrap(),
        "--phi",
        tabu_f.to_str().unwrap(),
    ]);
    let texto = String::from_utf8(salida.stdout).unwrap();
    let mut lineas = texto.lines();
    assert_eq!(lineas.next().unwrap(), "z,objetivo");
    let csv_tabu = std::fs::read_to_string(&tabu_f).unwrap();
    let mut n = 0;
    for (impreso, fila) in lineas.zip(csv_tabu.lines().skip(1)) {
        let (zi, oi) = impreso.split_once(',').unwrap();
        assert_eq!(
            zi.split_once('.').unwrap().1.len(),
            12,
            "12 decimales: {zi}"
        );
        let campos: Vec<&str> = fila.split(',').collect();
        let z: f64 = campos[1].parse().unwrap();
        let o: f64 = campos[2].parse().unwrap();
        assert!((zi.parse::<f64>().unwrap() - z).abs() < 1e-9);
        assert!((oi.parse::<f64>().unwrap() - o).abs() < 1e-9);
        n += 1;
    }
    assert_eq!(n, 2);
    // Un CSV de una columna `especie` (formato de patrón) también se evalúa.
    let i: InstanciaVal = serde_json::from_str(&std::fs::read_to_string(&inst_f).unwrap()).unwrap();
    let phi: Vec<u8> = (0..49).map(|u| (u % 10) as u8 + 1).collect();
    let mut patron = String::from("especie\n");
    for &c in &phi {
        patron.push_str(&format!("{}\n", c as i32 - 1));
    }
    let patron_f = res.join("patron.csv");
    std::fs::write(&patron_f, patron).unwrap();
    let salida = reto(&[
        "evaluar-val",
        "--inst",
        inst_f.to_str().unwrap(),
        "--phi",
        patron_f.to_str().unwrap(),
    ]);
    let texto = String::from_utf8(salida.stdout).unwrap();
    let (zi, oi) = texto.lines().nth(1).unwrap().split_once(',').unwrap();
    let (z, o) = evaluar(&i, &phi);
    assert!((zi.parse::<f64>().unwrap() - z).abs() < 1e-11);
    assert!((oi.parse::<f64>().unwrap() - o).abs() < 1e-11);
}
