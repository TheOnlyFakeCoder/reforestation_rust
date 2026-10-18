//! Binario del reto CONAFOR (Entregable 2).
//!
//! Uso: `reto <subcomando> [--datos DIR] [--resultados DIR]`
//!   datos   escribe `DIR/*.csv` con los datos del reto
//!   malla   |V|, D_tot y pares por anillo de 1 ha y de cada polígono; exporta W
//!   paso1   barrido de γ y patrón base [--instancias 1ha,map_27|todas] [--solo-gamma-star]

use std::path::Path;
use std::process::ExitCode;

use reforestation::competencia::matriz_w;
use reforestation::config::Config;
use reforestation::datos::Datos;
use reforestation::paso1::{barrido_gamma, mejor_fila, FilaPaso1, Instancia};
use reforestation::reportes::{
    escribir_alcance, escribir_anillos, escribir_barrido_gamma, escribir_composicion_paso1,
    escribir_convergencia, escribir_patron, escribir_resumen_gamma, escribir_resumen_malla,
    escribir_w, resumen_malla,
};

/// Configuración compartida con Python; si no existe se usan los valores por defecto.
const CONFIG_POR_DEFECTO: &str = "config/default.json";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match ejecutar(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Valor de `nombre VALOR`; error si la opción aparece sin valor.
fn opcion(args: &[String], nombre: &str, por_defecto: &str) -> Result<String, String> {
    match args.iter().position(|a| a == nombre) {
        None => Ok(por_defecto.to_string()),
        Some(i) => args
            .get(i + 1)
            .filter(|valor| !valor.starts_with("--"))
            .cloned()
            .ok_or_else(|| format!("{nombre} requiere un valor")),
    }
}

const OPCIONES: [&str; 5] = ["--datos", "--resultados", "--config", "--set", "--instancias"];
/// Opciones sin valor.
const BANDERAS: [&str; 1] = ["--solo-gamma-star"];

/// Tras el subcomando sólo se aceptan pares `--opción valor` y banderas conocidas.
fn validar_opciones(args: &[String]) -> Result<(), String> {
    let mut resto = args.iter().skip(2);
    while let Some(nombre) = resto.next() {
        if BANDERAS.contains(&nombre.as_str()) {
            continue;
        }
        if !OPCIONES.contains(&nombre.as_str()) {
            return Err(format!("opción desconocida: {nombre}"));
        }
        match resto.next() {
            Some(valor) if !valor.starts_with("--") => {}
            _ => return Err(format!("{nombre} requiere un valor")),
        }
    }
    Ok(())
}

/// Se lee sin validar, se aplican todos los --set y se valida una sola vez.
fn cargar_config(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = match args.windows(2).find(|par| par[0] == "--config") {
        Some(par) => Config::leer(Path::new(&par[1]))?,
        None if Path::new(CONFIG_POR_DEFECTO).exists() => {
            Config::leer(Path::new(CONFIG_POR_DEFECTO))?
        }
        None => Config::default(),
    };
    let asignaciones: Vec<&str> = args
        .windows(2)
        .filter(|par| par[0] == "--set")
        .map(|par| par[1].as_str())
        .collect();
    config.aplicar_todas(&asignaciones)?;
    Ok(config)
}

fn ejecutar(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    validar_opciones(args)?;
    let datos_dir = opcion(args, "--datos", "data")?;
    let resultados_dir = opcion(args, "--resultados", "results")?;
    match args.get(1).map(String::as_str) {
        Some("datos") => {
            Datos::base().escribir(Path::new(&datos_dir))?;
            println!("datos escritos en {datos_dir}");
            Ok(())
        }
        Some("malla") => {
            let config = cargar_config(args)?;
            malla(Path::new(&datos_dir), Path::new(&resultados_dir), &config)
        }
        Some("paso1") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            let solo_star = args.iter().any(|a| a == "--solo-gamma-star");
            paso1(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
                solo_star,
            )
        }
        otro => Err(format!("subcomando desconocido: {}", otro.unwrap_or("")).into()),
    }
}

fn malla(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    std::fs::create_dir_all(resultados_dir)?;

    let filas = resumen_malla(&datos, config);
    escribir_resumen_malla(&resultados_dir.join("malla_resumen.csv"), &filas)?;
    escribir_anillos(&resultados_dir.join("malla_anillos.csv"), &filas, config.d)?;
    escribir_alcance(
        &resultados_dir.join("alcance_h.csv"),
        &datos.especies,
        config,
        &filas[0].anillos_m,
    )?;
    let w = matriz_w(&datos.especies, config.kappa);
    escribir_w(&resultados_dir.join("W.csv"), &datos.especies, &w)?;

    println!("d = {:.4} m", config.d);
    for fila in filas.iter().take(2) {
        println!(
            "{:<8} A = {:.2} ha  {} x {} ({:.1} x {:.1} m)  |V| = {}  round(658A) = {}  \
             excedente = {}  D_tot = {}",
            fila.instancia,
            fila.area,
            fila.columnas,
            fila.filas,
            fila.ancho_m,
            fila.alto_m,
            fila.n_pos,
            fila.objetivo,
            fila.excedente,
            fila.d_tot
        );
        for (k, (distancia, pares)) in fila.anillos_m.iter().zip(&fila.pares).enumerate() {
            println!(
                "    anillo {}: {:.3} m ({:.3}·d)  {} pares",
                k + 1,
                distancia,
                distancia / config.d,
                pares
            );
        }
    }
    Ok(())
}

fn paso1(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
    solo_star: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    std::fs::create_dir_all(resultados_dir)?;
    let mut pedidas: Vec<(String, f64)> = Vec::new();
    for nombre in instancias.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        if nombre == "todas" {
            pedidas.push(("1ha".to_string(), 1.0));
            pedidas.extend(datos.poligonos.iter().map(|p| (format!("map_{}", p.map_id), p.area)));
        } else if nombre == "1ha" {
            pedidas.push((nombre.to_string(), 1.0));
        } else {
            let poligono = nombre
                .strip_prefix("map_")
                .and_then(|m| m.parse::<u32>().ok())
                .and_then(|m| datos.poligono_por_map_id(m))
                .ok_or_else(|| format!("instancia desconocida: {nombre}"))?;
            pedidas.push((nombre.to_string(), poligono.area));
        }
    }
    if pedidas.is_empty() {
        return Err("--instancias no nombra ninguna instancia".into());
    }

    let mut gammas = if solo_star { vec![] } else { config.gammas.clone() };
    if !gammas.contains(&config.gamma_star) {
        gammas.push(config.gamma_star);
    }
    let semillas: Vec<u64> = (0..config.n_semillas as u64).map(|k| config.semilla + k).collect();
    let params = config.tabu.params(config.semilla);

    let inicio = std::time::Instant::now();
    let mut todas: Vec<FilaPaso1> = Vec::new();
    let mut elegidas: Vec<(Instancia, FilaPaso1)> = Vec::new();
    for (nombre, area) in &pedidas {
        let instancia =
            Instancia::nueva(nombre, *area, &datos, config, config.m_escala, config.usar_s2);
        let filas = barrido_gamma(&instancia, &gammas, &semillas, &params);
        let estrella = mejor_fila(filas.iter().filter(|f| f.gamma == config.gamma_star))
            .expect("γ* siempre se corre")
            .clone();
        println!(
            "{nombre}: D_tot = {}, γ* = {}: Z_opt = {:.2} (Z/planta {:.4}), D = {:?}",
            instancia.d_tot, config.gamma_star, estrella.z_opt, estrella.z_opt_por_planta, estrella.d
        );
        todas.extend(filas);
        elegidas.push((instancia, estrella));
    }

    escribir_barrido_gamma(&resultados_dir.join("paso1_barrido_gamma.csv"), &todas)?;
    escribir_resumen_gamma(&resultados_dir.join("paso1_resumen_gamma.csv"), &todas)?;
    let pares: Vec<(&Instancia, &FilaPaso1)> = elegidas.iter().map(|(i, f)| (i, f)).collect();
    escribir_composicion_paso1(&resultados_dir.join("paso1_composicion.csv"), &pares)?;
    for (instancia, fila) in &elegidas {
        escribir_patron(
            &resultados_dir.join(format!("paso1_patron_{}.csv", instancia.nombre)),
            &instancia.malla,
            &fila.phi,
        )?;
        escribir_convergencia(
            &resultados_dir.join(format!("convergencia_{}.csv", instancia.nombre)),
            fila,
        )?;
    }
    println!("paso 1: {:.1} s", inicio.elapsed().as_secs_f64());
    Ok(())
}
