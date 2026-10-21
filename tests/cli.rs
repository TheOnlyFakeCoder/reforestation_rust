//! Pruebas del binario `reto` de punta a punta, en directorios temporales.

use std::path::PathBuf;
use std::process::Command;

use reforestation::datos::Datos;

fn dir_temporal(nombre: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("reto_cli_{}_{}", nombre, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn reto(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_reto"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn datos_escribe_los_csv_del_reto() {
    let dir = dir_temporal("datos");
    let salida = reto(&["datos", "--datos", dir.to_str().unwrap()]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(
        dir.join("especies.csv").exists(),
        "el binario `reto datos --datos DIR` no escribió DIR/especies.csv"
    );
    assert_eq!(Datos::cargar(&dir).unwrap(), Datos::base());
}

#[test]
fn subcomando_desconocido_termina_con_error_sin_escribir() {
    let dir = dir_temporal("desconocido");
    let salida = reto(&["dato", "--datos", dir.to_str().unwrap()]);
    assert!(!salida.status.success(), "`reto dato` debería fallar");
    assert!(String::from_utf8_lossy(&salida.stderr).contains("dato"));
    assert!(!dir.join("especies.csv").exists());
}

#[test]
fn opcion_sin_valor_termina_con_error_sin_escribir() {
    let raiz = dir_temporal("sin_valor");
    let salida = Command::new(env!("CARGO_BIN_EXE_reto"))
        .args(["datos", "--datos"])
        .current_dir(&raiz)
        .output()
        .unwrap();
    assert!(
        !salida.status.success(),
        "`reto datos --datos` (sin valor) debería fallar"
    );
    assert!(String::from_utf8_lossy(&salida.stderr).contains("--datos"));
    assert!(
        !raiz.join("data").exists(),
        "no debe escribir en la carpeta por defecto"
    );
}

#[test]
fn opcion_desconocida_termina_con_error_sin_escribir() {
    let datos = dir_temporal("opcion_desconocida");
    let resultados = datos.join("resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--sett",
        "kappa=0.4",
    ]);
    assert!(
        !salida.status.success(),
        "`--sett` (typo de --set) debería fallar"
    );
    assert!(String::from_utf8_lossy(&salida.stderr).contains("--sett"));
    assert!(
        !resultados.exists(),
        "no debe escribir resultados con opciones inválidas"
    );
}

#[test]
fn malla_exporta_resumen_anillos_y_w() {
    let datos = dir_temporal("malla_datos");
    let resultados = dir_temporal("malla_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
    ]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );

    let resumen = std::fs::read_to_string(resultados.join("malla_resumen.csv")).unwrap();
    assert_eq!(resumen.lines().count(), 32);
    let p1 = resumen.lines().find(|l| l.starts_with("map_27,")).unwrap();
    assert!(p1.starts_with("map_27,27,1.28,26,33,alterna,"), "{p1}");
    assert!(p1.contains(",842,842,0,842,"), "{p1}");

    let anillos = std::fs::read_to_string(resultados.join("malla_anillos.csv")).unwrap();
    assert_eq!(anillos.lines().count(), 1 + 31 * 3);

    let alcance = std::fs::read_to_string(resultados.join("alcance_h.csv"))
        .expect("`reto malla` debe exportar alcance_h.csv");
    assert_eq!(alcance.lines().count(), 1 + 6 * 3);

    let w = std::fs::read_to_string(resultados.join("W.csv")).unwrap();
    let filas: Vec<Vec<&str>> = w.lines().map(|l| l.split(',').collect()).collect();
    assert_eq!((filas.len(), filas[0].len()), (11, 11));
    assert_eq!(filas[1][9].parse::<f64>().unwrap(), 0.0);

    let texto = String::from_utf8_lossy(&salida.stdout);
    // Cambio de requisito (usuario, 2026-10-17): malla rectangular con D_tot = round(658·A);
    // P1 pasa de 837 a 842 posiciones, así que "837" ya no es el comportamiento esperado.
    assert!(texto.contains("658") && texto.contains("842"), "{texto}");
    assert!(
        texto.contains("26 x 33") && texto.contains("excedente = 0"),
        "la salida debe mostrar columnas x filas y el excedente: {texto}"
    );
}

#[test]
fn set_factor_radios_1_25_agrega_dos_anillos() {
    let datos = dir_temporal("set_datos");
    let resultados = dir_temporal("set_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--set",
        "factor_radios=1.25",
    ]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let anillos = std::fs::read_to_string(resultados.join("malla_anillos.csv")).unwrap();
    assert_eq!(anillos.lines().count(), 1 + 31 * 5);
}

#[test]
fn sin_config_lee_config_default_json_del_directorio_actual() {
    let raiz = dir_temporal("config_default");
    std::fs::create_dir_all(raiz.join("config")).unwrap();
    std::fs::write(
        raiz.join("config/default.json"),
        r#"{"factor_radios": 1.25}"#,
    )
    .unwrap();
    let corre = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_reto"))
            .args(args)
            .current_dir(&raiz)
            .output()
            .unwrap()
    };
    assert!(corre(&["datos"]).status.success());
    let salida = corre(&["malla"]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let anillos = std::fs::read_to_string(raiz.join("results/malla_anillos.csv")).unwrap();
    assert_eq!(
        anillos.lines().count(),
        1 + 31 * 5,
        "`reto malla` sin --config no leyó config/default.json (factor_radios = 1.25)"
    );
}

#[test]
fn config_json_cambia_kappa_de_w() {
    let datos = dir_temporal("config_datos");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let config = datos.join("config.json");
    std::fs::write(&config, r#"{"kappa": 0.4}"#).unwrap();
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        datos.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let w = std::fs::read_to_string(datos.join("W.csv")).unwrap();
    let fila_lechuguilla: Vec<&str> = w.lines().nth(1).unwrap().split(',').collect();
    // κ·S con S(lechuguilla, salmiana) = 0.93400 (calculado a mano): 0.4 · 0.934 = 0.3736
    let w01: f64 = fila_lechuguilla[2].parse().unwrap();
    assert!(
        (w01 - 0.3736).abs() < 1e-4,
        "`reto malla --config ARCHIVO` ignoró el archivo (kappa = 0.4): W[0][1] = {w01}, se esperaba 0.3736"
    );
}

#[test]
fn la_validacion_corre_despues_de_todos_los_set() {
    let datos = dir_temporal("validar_al_final");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let config = datos.join("config.json");
    std::fs::write(&config, r#"{"epsilon": 1.5, "kappa": -1}"#).unwrap();
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        datos.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--set",
        "epsilon=0.1",
        "--set",
        "kappa=0.6",
    ]);
    assert!(
        salida.status.success(),
        "los --set corrigen el archivo y se valida al final: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

const TABU_CORTO: &str = r#"tabu={"candidatos":20,"tenure":5,"max_iter":200,"max_sin_mejora":200,"prob_intercambio":0.5}"#;

fn encabezado_de(dir: &std::path::Path, archivo: &str) -> String {
    let texto = std::fs::read_to_string(dir.join(archivo))
        .unwrap_or_else(|e| panic!("`reto paso1` no escribió {archivo}: {e}"));
    texto.lines().next().unwrap().to_string()
}

#[test]
fn paso1_escribe_los_csv_con_los_encabezados_exactos() {
    let datos = dir_temporal("paso1_datos");
    let resultados = dir_temporal("paso1_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let salida = reto(&[
        "paso1",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--instancias",
        "1ha",
        "--set",
        TABU_CORTO,
        "--set",
        "gammas=[0.0,1.0]",
        "--set",
        "n_semillas=2",
    ]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let d: String = (0..10).map(|j| format!("D_{j},")).collect();
    assert_eq!(
        encabezado_de(&resultados, "paso1_barrido_gamma.csv"),
        format!(
            "instancia,map_id,gamma,semilla,{d}d_tot,suma_theta_d,suma_c_d,z_paso1,z_opt,\
             z_opt_por_planta,tiempo_paso1_s,iter_paso1,tiempo_opt_s,iter_opt,\
             motivo_paro_paso1,motivo_paro_opt"
        )
    );
    assert_eq!(
        encabezado_de(&resultados, "paso1_resumen_gamma.csv"),
        format!(
            "instancia,map_id,gamma,{d}suma_theta_d,suma_c_d,z_opt_media,z_opt_min,z_opt_max,\
             z_opt_por_planta_media,tiempo_medio_s,iter_media,z_opt_sd,paros_tope,paros_sin_mejora"
        )
        .replace(",,", ",")
    );
    let cols = |p: &str| {
        (0..10)
            .map(|j| format!("{p}_{j}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    assert_eq!(
        encabezado_de(&resultados, "paso1_composicion.csv"),
        format!(
            "instancia,map_id,area_ha,d_tot,{},{},{},{}",
            cols("D"),
            cols("Dnom"),
            cols("L"),
            cols("U")
        )
    );
    assert_eq!(
        encabezado_de(&resultados, "paso1_patron_1ha.csv"),
        "pos,fila,col,x,y,especie"
    );
    assert_eq!(
        encabezado_de(&resultados, "convergencia_1ha.csv"),
        "iteracion,actual,mejor"
    );
    let barrido = std::fs::read_to_string(resultados.join("paso1_barrido_gamma.csv")).unwrap();
    assert!(
        barrido.lines().count() >= 5,
        "2 γ × 2 semillas + encabezado"
    );
    for linea in barrido.lines().skip(1) {
        let campos: Vec<&str> = linea.split(',').collect();
        let n = campos.len();
        for motivo in &campos[n - 2..] {
            assert!(["tope", "sin_mejora"].contains(motivo), "{linea}");
        }
    }
    let resumen = std::fs::read_to_string(resultados.join("paso1_resumen_gamma.csv")).unwrap();
    for linea in resumen.lines().skip(1) {
        let campos: Vec<&str> = linea.split(',').collect();
        let n = campos.len();
        let tope: usize = campos[n - 2].parse().unwrap();
        let sin: usize = campos[n - 1].parse().unwrap();
        assert_eq!(tope + sin, 4, "2 búsquedas × 2 semillas: {linea}");
        assert!(campos[n - 3].parse::<f64>().unwrap() >= 0.0);
    }
    let patron = std::fs::read_to_string(resultados.join("paso1_patron_1ha.csv")).unwrap();
    assert_eq!(patron.lines().count(), 1 + 658);
    let comp = std::fs::read_to_string(resultados.join("paso1_composicion.csv")).unwrap();
    assert_eq!(comp.lines().count(), 2);
    assert!(comp.lines().nth(1).unwrap().starts_with("1ha,0,1,658,"));
}

#[test]
fn paso1_solo_gamma_star_corre_un_solo_gamma_y_valida_despues_de_set() {
    let datos = dir_temporal("paso1s_datos");
    let resultados = dir_temporal("paso1s_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let base = |extra: &[&'static str]| {
        let mut args = vec![
            "paso1",
            "--datos",
            datos.to_str().unwrap(),
            "--resultados",
            resultados.to_str().unwrap(),
            "--instancias",
            "1ha",
            "--solo-gamma-star",
            "--set",
            TABU_CORTO,
            "--set",
            "n_semillas=2",
        ];
        args.extend_from_slice(extra);
        reto(&args)
    };
    let salida = base(&[]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(
        !resultados.join("paso1_resumen_gamma.csv").exists(),
        "no pisa el barrido"
    );
    assert!(
        !resultados.join("paso1_barrido_gamma.csv").exists(),
        "no pisa el barrido"
    );
    let estrella = std::fs::read_to_string(resultados.join("paso1_gamma_star.csv")).unwrap();
    assert_eq!(estrella.lines().count(), 3, "γ* × 2 semillas");
    // Un --set inválido (n_semillas=0) falla aunque venga después de los demás.
    let salida = base(&["--set", "n_semillas=0"]);
    assert!(!salida.status.success());
    assert!(String::from_utf8_lossy(&salida.stderr).contains("n_semillas"));
}

#[test]
fn montecarlo_escribe_escenarios_y_resumenes() {
    let datos = dir_temporal("mc_datos");
    let resultados = dir_temporal("mc_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let comunes = |sub: &'static str| {
        vec![
            sub,
            "--datos",
            datos.to_str().unwrap(),
            "--resultados",
            resultados.to_str().unwrap(),
            "--instancias",
            "1ha",
            "--set",
            TABU_CORTO,
        ]
    };
    let mut args = comunes("paso1");
    args.extend(["--solo-gamma-star", "--set", "n_semillas=1"]);
    assert!(reto(&args).status.success());
    std::fs::write(
        resultados.join("mc_params.json"),
        r#"{"mu":129.0,"sigma":3.3,"p_hat":[0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1]}"#,
    )
    .unwrap();
    let mut args = comunes("montecarlo");
    args.extend([
        "--set",
        "n_escenarios=20",
        "--set",
        "n_escenarios_paso3=20",
        "--set",
        "escenarios_paso3_por_instancia={}",
    ]);
    let salida = reto(&args);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let x = std::fs::read_to_string(resultados.join("escenarios_X_1ha.csv")).unwrap();
    assert_eq!(x.lines().count(), 21);
    assert!(x.starts_with("omega,T,X_0,"));
    let esp = std::fs::read_to_string(resultados.join("mc_resumen_especies.csv")).unwrap();
    assert_eq!(esp.lines().count(), 1 + 2 * 10);
    let costo = std::fs::read_to_string(resultados.join("mc_resumen_costo.csv")).unwrap();
    assert_eq!(costo.lines().count(), 3);
}

#[test]
fn paso3_y_practica_escriben_el_motivo_de_paro() {
    let datos = dir_temporal("motivo_datos");
    let resultados = dir_temporal("motivo_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let comunes = |sub: &'static str| {
        vec![
            sub,
            "--datos",
            datos.to_str().unwrap(),
            "--resultados",
            resultados.to_str().unwrap(),
            "--instancias",
            "1ha",
            "--set",
            TABU_CORTO,
        ]
    };
    let mut args = comunes("paso1");
    args.extend(["--solo-gamma-star", "--set", "n_semillas=1"]);
    assert!(reto(&args).status.success());
    std::fs::write(
        resultados.join("mc_params.json"),
        r#"{"mu":129.0,"sigma":3.3,"p_hat":[0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1]}"#,
    )
    .unwrap();
    let q: Vec<String> = [42, 196, 42, 42, 49, 38, 73, 64, 86, 26]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let cabecera: String = (0..10).map(|j| format!(",q_{j}")).collect();
    std::fs::write(
        resultados.join("paso2_compra.csv"),
        format!(
            "instancia,epsilon,pi_factor{cabecera}
1ha,0.1,2.0,{}
",
            q.join(",")
        ),
    )
    .unwrap();
    let mut args = comunes("paso3");
    args.extend([
        "--set",
        "n_escenarios_paso3=4",
        "--set",
        "escenarios_paso3_por_instancia={}",
    ]);
    let salida = reto(&args);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let esc = encabezado_de(&resultados, "paso3_escenarios.csv");
    assert!(esc.contains(",iteraciones,motivo_paro,Z_nivel1,"), "{esc}");
    assert!(esc.ends_with(",Z_nivel5,Z_otro"), "{esc}");
    let texto = std::fs::read_to_string(resultados.join("paso3_escenarios.csv")).unwrap();
    for linea in texto.lines().skip(1) {
        let campos: Vec<&str> = linea.split(',').collect();
        let motivo = campos[campos.len() - 7];
        assert!(["tope", "sin_mejora"].contains(&motivo), "{linea}");
        let z: f64 = campos[7].parse().unwrap();
        let niveles: f64 = campos[campos.len() - 6..]
            .iter()
            .map(|c| c.parse::<f64>().unwrap())
            .sum();
        assert!((z - niveles).abs() < 1e-6, "{linea}");
    }
    let trans = std::fs::read_to_string(resultados.join("paso3_transicion_1ha.csv")).unwrap();
    assert_eq!(trans.lines().count(), 201);
    let plantas = std::fs::read_to_string(resultados.join("paso3_plantas_1ha.csv")).unwrap();
    assert_eq!(plantas.lines().count(), 11);
    let res = encabezado_de(&resultados, "paso3_resumen.csv");
    assert!(res.ends_with(",paros_tope,paros_sin_mejora"), "{res}");
    let texto = std::fs::read_to_string(resultados.join("paso3_resumen.csv")).unwrap();
    let campos: Vec<&str> = texto.lines().nth(1).unwrap().split(',').collect();
    let n = campos.len();
    let (tope, sin): (usize, usize) = (
        campos[n - 2].parse().unwrap(),
        campos[n - 1].parse().unwrap(),
    );
    assert_eq!(tope + sin, 4);

    let salida = reto(&comunes("practica"));
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let texto = std::fs::read_to_string(resultados.join("practica_distribucion.csv")).unwrap();
    assert!(texto
        .lines()
        .next()
        .unwrap()
        .ends_with(",Z_otro,motivo_paro"));
    for linea in texto.lines().skip(1) {
        let variante = linea.split(',').nth(1).unwrap();
        let motivo = linea.rsplit(',').next().unwrap();
        if variante.starts_with("tabu_") {
            assert!(["tope", "sin_mejora"].contains(&motivo), "{linea}");
        } else {
            assert_eq!(motivo, "", "{linea}");
        }
    }
}

#[test]
fn calibrar_escribe_el_csv_con_el_encabezado_exacto() {
    let datos = dir_temporal("calibrar_datos");
    let resultados = dir_temporal("calibrar_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let salida = reto(&[
        "calibrar",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--instancias",
        "1ha",
        "--candidatos",
        "16,24",
        "--tenures",
        "5",
        "--semillas",
        "2",
        "--set",
        TABU_CORTO,
    ]);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert_eq!(
        encabezado_de(&resultados, "calibracion_tabu.csv"),
        "instancia,candidatos,tenure,n_semillas,z_opt_media,z_opt_sd,z_opt_mejor,\
         objetivo_media,tiempo_medio_s,iter_media,frac_sin_mejora"
    );
    let texto = std::fs::read_to_string(resultados.join("calibracion_tabu.csv")).unwrap();
    assert_eq!(texto.lines().count(), 3);
    assert!(String::from_utf8_lossy(&salida.stdout).contains("elegida"));
}

#[test]
fn paso3_usa_los_escenarios_por_instancia() {
    let datos = dir_temporal("por_inst_datos");
    let resultados = dir_temporal("por_inst_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()])
        .status
        .success());
    let comunes = |sub: &'static str| {
        vec![
            sub,
            "--datos",
            datos.to_str().unwrap(),
            "--resultados",
            resultados.to_str().unwrap(),
            "--instancias",
            "1ha",
            "--set",
            TABU_CORTO,
        ]
    };
    let mut args = comunes("paso1");
    args.extend(["--solo-gamma-star", "--set", "n_semillas=1"]);
    assert!(reto(&args).status.success());
    std::fs::write(
        resultados.join("mc_params.json"),
        r#"{"mu":129.0,"sigma":3.3,"p_hat":[0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1]}"#,
    )
    .unwrap();
    let q: Vec<String> = [42, 196, 42, 42, 49, 38, 73, 64, 86, 26]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let cabecera: String = (0..10).map(|j| format!(",q_{j}")).collect();
    std::fs::write(
        resultados.join("paso2_compra.csv"),
        format!(
            "instancia,epsilon,pi_factor{cabecera}
1ha,0.1,2.0,{}
",
            q.join(",")
        ),
    )
    .unwrap();
    let mut args = comunes("paso3");
    args.extend([
        "--set",
        "n_escenarios=20",
        "--set",
        "n_escenarios_paso3=5",
        "--set",
        r#"escenarios_paso3_por_instancia={"1ha":12}"#,
    ]);
    let salida = reto(&args);
    assert!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let texto = std::fs::read_to_string(resultados.join("paso3_resumen.csv")).unwrap();
    let campos: Vec<&str> = texto.lines().nth(1).unwrap().split(',').collect();
    assert_eq!(campos[0], "1ha");
    assert_eq!(campos[2], "12", "{texto}");
}
