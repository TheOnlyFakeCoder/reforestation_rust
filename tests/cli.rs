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
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
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
    assert!(!salida.status.success(), "`reto datos --datos` (sin valor) debería fallar");
    assert!(String::from_utf8_lossy(&salida.stderr).contains("--datos"));
    assert!(!raiz.join("data").exists(), "no debe escribir en la carpeta por defecto");
}

#[test]
fn opcion_desconocida_termina_con_error_sin_escribir() {
    let datos = dir_temporal("opcion_desconocida");
    let resultados = datos.join("resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()]).status.success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--sett",
        "kappa=0.4",
    ]);
    assert!(!salida.status.success(), "`--sett` (typo de --set) debería fallar");
    assert!(String::from_utf8_lossy(&salida.stderr).contains("--sett"));
    assert!(!resultados.exists(), "no debe escribir resultados con opciones inválidas");
}

#[test]
fn malla_exporta_resumen_anillos_y_w() {
    let datos = dir_temporal("malla_datos");
    let resultados = dir_temporal("malla_resultados");
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()]).status.success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
    ]);
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));

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
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()]).status.success());
    let salida = reto(&[
        "malla",
        "--datos",
        datos.to_str().unwrap(),
        "--resultados",
        resultados.to_str().unwrap(),
        "--set",
        "factor_radios=1.25",
    ]);
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
    let anillos = std::fs::read_to_string(resultados.join("malla_anillos.csv")).unwrap();
    assert_eq!(anillos.lines().count(), 1 + 31 * 5);
}

#[test]
fn sin_config_lee_config_default_json_del_directorio_actual() {
    let raiz = dir_temporal("config_default");
    std::fs::create_dir_all(raiz.join("config")).unwrap();
    std::fs::write(raiz.join("config/default.json"), r#"{"factor_radios": 1.25}"#).unwrap();
    let corre = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_reto"))
            .args(args)
            .current_dir(&raiz)
            .output()
            .unwrap()
    };
    assert!(corre(&["datos"]).status.success());
    let salida = corre(&["malla"]);
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
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
    assert!(reto(&["datos", "--datos", datos.to_str().unwrap()]).status.success());
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
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
    let w = std::fs::read_to_string(datos.join("W.csv")).unwrap();
    let fila_lechuguilla: Vec<&str> = w.lines().nth(1).unwrap().split(',').collect();
    // κ·S con S(lechuguilla, salmiana) = 0.93400 (calculado a mano): 0.4 · 0.934 = 0.3736
    let w01: f64 = fila_lechuguilla[2].parse().unwrap();
    assert!(
        (w01 - 0.3736).abs() < 1e-4,
        "`reto malla --config ARCHIVO` ignoró el archivo (kappa = 0.4): W[0][1] = {w01}, se esperaba 0.3736"
    );
}
